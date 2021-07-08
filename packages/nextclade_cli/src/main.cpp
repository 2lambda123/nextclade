#include <fmt/format.h>
#include <nextalign/nextalign.h>
#include <nextclade/nextclade.h>
#include <nextclade_common/datasets.h>

#include "description.h"
#include "generated/cli.h"
#include "io/Logger.h"
#include "io/format.h"
#include "io/getPaths.h"
#include "io/openOutputFile.h"
#include "io/parseGeneMapGffFile.h"
#include "io/parseRefFastaFile.h"
#include "io/readFile.h"
#include "run.h"

namespace Nextclade {

  int main(int argc, char *argv[]) {
    Logger logger{Logger::Options{.linePrefix = "Nextclade", .verbosity = Logger::Verbosity::warn}};

    //  const auto datasetsJson = Nextclade::fetchDatasetsJson();

    //  const std::string thisVersion = Nextclade::getVersion();

    //  fmt::print("All datasets:\n\n");
    //  fmt::print("{:s}\n", formatDatasets(datasetsJson.datasets));
    //
    //  fmt::print("Latest datasets compatible with this version of Nextclade ({:s}):\n\n", thisVersion);
    //  fmt::print("{:s}\n", formatDatasets(Nextclade::getLatestCompatibleDatasets(datasetsJson.datasets, thisVersion)));
    //
    //  fmt::print("Latest datasets:\n\n");
    //  fmt::print("{:s}\n", formatDatasets(Nextclade::getLatestDatasets(datasetsJson.datasets)));
    //
    //  fmt::print("Datasets compatible with this version of Nextclade ({:s}):\n\n", thisVersion);
    //  fmt::print("{:s}\n", formatDatasets(Nextclade::getCompatibleDatasets(datasetsJson.datasets, thisVersion)));

    //  fflush(stdout);

    //  std::exit(0);

    try {
      const auto cliParams = parseCommandLine(argc, argv, PROJECT_DESCRIPTION);

      auto verbosity = Logger::convertVerbosity(cliParams.verbosity);
      if (cliParams.verbose) {
        verbosity = Logger::Verbosity::info;
      }

      if (cliParams.silent) {
        verbosity = Logger::Verbosity::silent;
      }

      logger.setVerbosity(verbosity);
      logger.info(formatCliParams(cliParams));

      const auto refData = parseRefFastaFile(cliParams.inputRootSeq);
      const auto shouldWriteReference = cliParams.writeReference;
      logger.info(formatRef(refData, shouldWriteReference));

      GeneMap geneMap;
      std::set<std::string> genes;
      if (cliParams.inputGeneMap) {
        geneMap = parseGeneMapGffFile(*cliParams.inputGeneMap);

        if (!cliParams.genes || cliParams.genes->empty()) {
          // If `--genes` are omitted or empty, use all genes in the gene map
          std::transform(geneMap.cbegin(), geneMap.cend(), std::inserter(genes, genes.end()),
            [](const auto &it) { return it.first; });
        } else {
          genes = parseGenes(*cliParams.genes);
        }

        validateGenes(genes, geneMap);

        const GeneMap geneMapFull = geneMap;
        geneMap = filterGeneMap(genes, geneMap);
        logger.info(formatGeneMap(geneMapFull, genes));
      }

      //      if (!genes.empty()) {
      //        // penaltyGapOpenOutOfFrame > penaltyGapOpenInFrame > penaltyGapOpen
      //        const auto isInFrameGreater = cliParams.alignment.penaltyGapOpenInFrame > cliParams.alignment.penaltyGapOpen;
      //        const auto isOutOfFrameEvenGreater =
      //          cliParams.alignment.penaltyGapOpenOutOfFrame > cliParams.alignment.penaltyGapOpenInFrame;
      //        if (!(isInFrameGreater && isOutOfFrameEvenGreater)) {
      //          throw ErrorCliOptionInvalidValue(fmt::format(
      //            "Should verify the condition `--penalty-gap-open-out-of-frame` > `--penalty-gap-open-in-frame` > "
      //            "`--penalty-gap-open`, but got {:d} > {:d} > {:d}, which is false",
      //            cliParams.alignment.penaltyGapOpenOutOfFrame, cliParams.alignment.penaltyGapOpenInFrame,
      //            cliParams.alignment.penaltyGapOpen));
      //        }
      //      }

      std::ifstream fastaFile(cliParams.inputFasta);
      auto inputFastaStream = makeFastaStream(fastaFile, cliParams.inputFasta);
      if (!fastaFile.good()) {
        logger.error("Error: unable to read \"{:s}\"", cliParams.inputFasta);
        std::exit(1);
      }

      const auto qcJsonString = readFile(cliParams.inputQcConfig);
      const auto qcRulesConfig = Nextclade::parseQcConfig(qcJsonString);
      if (!Nextclade::isQcConfigVersionRecent(qcRulesConfig)) {
        logger.warn(
          "The QC configuration file \"{:s}\" version ({:s}) is older than the version of Nextclade ({:s}). You might "
          "be "
          "missing out on new features. It is recommended to download the latest configuration file. Alternatively, to "
          "silence this warning, add/change property \"schemaVersion\": \"{:s}\" in your file.",
          cliParams.inputQcConfig, qcRulesConfig.schemaVersion, Nextclade::getVersion(), Nextclade::getVersion());
      }

      const auto treeString = readFile(cliParams.inputTree);

      std::vector<Nextclade::PcrPrimer> pcrPrimers;
      if (cliParams.inputPcrPrimers) {
        const auto pcrPrimersCsvString = readFile(*cliParams.inputPcrPrimers);
        std::vector<std::string> warnings;
        pcrPrimers = Nextclade::parseAndConvertPcrPrimersCsv(pcrPrimersCsvString, *cliParams.inputPcrPrimers,
          refData.seq, warnings);
      }

      const auto paths = getPaths(cliParams, genes);
      logger.info(formatPaths(paths));

      auto outputJsonStream = openOutputFileMaybe(cliParams.outputJson);
      auto outputCsvStream = openOutputFileMaybe(cliParams.outputCsv);
      auto outputTsvStream = openOutputFileMaybe(cliParams.outputTsv);
      auto outputTreeStream = openOutputFileMaybe(cliParams.outputTree);

      std::ofstream outputFastaStream;
      openOutputFile(paths.outputFasta, outputFastaStream);

      std::ofstream outputInsertionsStream;
      openOutputFile(paths.outputInsertions, outputInsertionsStream);
      outputInsertionsStream << "seqName,insertions\n";

      std::ofstream outputErrorsFile;
      openOutputFile(paths.outputErrors, outputErrorsFile);
      outputErrorsFile << "seqName,errors,warnings,failedGenes\n";


      std::map<std::string, std::ofstream> outputGeneStreams;
      for (const auto &[geneName, outputGenePath] : paths.outputGenes) {
        auto result = outputGeneStreams.emplace(std::make_pair(geneName, std::ofstream{}));
        auto &outputGeneFile = result.first->second;
        openOutputFile(outputGenePath, outputGeneFile);
      }

      int parallelism = static_cast<int>(std::thread::hardware_concurrency());
      if (cliParams.jobs > 0) {
        tbb::global_control globalControl{tbb::global_control::max_allowed_parallelism,
          static_cast<size_t>(cliParams.jobs)};
        parallelism = cliParams.jobs;
      }

      bool inOrder = cliParams.inOrder;

      logger.info("\nParallelism: {:d}\n", parallelism);

      if (!cliParams.inputGeneMap) {
        logger.warn(
          "Warning: Parameter `--input-gene-map` was not specified. Without a gene map sequences will not be "
          "translated, there will be no peptides in output files, aminoacid mutations will not be detected and "
          "nucleotide sequence alignment will not be informed by codon boundaries.");
      } else if (geneMap.empty()) {
        logger.warn(
          "Warning: Provided gene map is empty. Sequences will not be translated, there will be no peptides in output "
          "files, aminoacid mutations will not be detected and nucleotide sequence alignment will not be informed by "
          "codon boundaries.");
      }

      constexpr const auto TABLE_WIDTH = 92;
      logger.info("\nSequences:");
      logger.info("{:s}", std::string(TABLE_WIDTH, '-'));
      logger.info("| {:5s} | {:40s} | {:7s} | {:7s} | {:7s} | {:7s} |",//
        "Index", "Seq. name", "A.score", "A.start", "A.end", "Insert." //
      );
      logger.info("{:s}\n", std::string(TABLE_WIDTH, '-'));

      try {
        run(parallelism, inOrder, inputFastaStream, refData, qcRulesConfig, treeString, pcrPrimers, geneMap, options,
          outputJsonStream, outputCsvStream, outputTsvStream, outputTreeStream, outputFastaStream,
          outputInsertionsStream, outputErrorsFile, outputGeneStreams, shouldWriteReference, logger);
      } catch (const std::exception &e) {
        logger.error("Error: {:>16s} |", e.what());
      }

      logger.info("{:s}", std::string(TABLE_WIDTH, '-'));
    } catch (const std::exception &e) {
      logger.error("Error: {:s}", e.what());
      std::exit(1);
    }
  }
}// namespace Nextclade

int main(int argc, char *argv[]) {
  return Nextclade::main(argc, argv);
}