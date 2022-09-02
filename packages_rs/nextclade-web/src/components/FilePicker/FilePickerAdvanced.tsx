import React, { useMemo } from 'react'

import { useTranslation } from 'react-i18next'
import { Col, Row } from 'reactstrap'
import { useRecoilState, useRecoilValue, useResetRecoilState } from 'recoil'

import {
  geneMapErrorAtom,
  primersCsvErrorAtom,
  qcConfigErrorAtom,
  refSeqErrorAtom,
  refTreeErrorAtom,
  virusPropertiesErrorAtom,
} from 'src/state/error.state'
import {
  geneMapInputAtom,
  primersCsvInputAtom,
  qcConfigInputAtom,
  refSeqInputAtom,
  refTreeInputAtom,
  virusPropertiesInputAtom,
} from 'src/state/inputs.state'

import { FileIconCsv, FileIconFasta, FileIconGff, FileIconJson } from 'src/components/Common/FileIcons'
import { FilePicker } from 'src/components/FilePicker/FilePicker'

export function FilePickerAdvanced() {
  const { t } = useTranslation()

  const [refSeq, setRefSeq] = useRecoilState(refSeqInputAtom)
  const refSeqError = useRecoilValue(refSeqErrorAtom)
  const resetRefSeq = useResetRecoilState(refSeqInputAtom)

  const [geneMap, setGeneMap] = useRecoilState(geneMapInputAtom)
  const geneMapError = useRecoilValue(geneMapErrorAtom)
  const resetGeneMap = useResetRecoilState(geneMapInputAtom)

  const [refTree, setRefTree] = useRecoilState(refTreeInputAtom)
  const refTreeError = useRecoilValue(refTreeErrorAtom)
  const resetRefTree = useResetRecoilState(refTreeInputAtom)

  const [qcConfig, setQcConfig] = useRecoilState(qcConfigInputAtom)
  const qcConfigError = useRecoilValue(qcConfigErrorAtom)
  const resetQcConfig = useResetRecoilState(qcConfigInputAtom)

  const [virusProperties, setVirusProperties] = useRecoilState(virusPropertiesInputAtom)
  const virusPropertiesError = useRecoilValue(virusPropertiesErrorAtom)
  const resetVirusProperties = useResetRecoilState(virusPropertiesInputAtom)

  const [primersCsv, setPrimersCsv] = useRecoilState(primersCsvInputAtom)
  const primersCsvError = useRecoilValue(primersCsvErrorAtom)
  const resetPrimersCsv = useResetRecoilState(primersCsvInputAtom)

  const iconCsv = useMemo(() => <FileIconCsv size={30} />, [])
  const iconFasta = useMemo(() => <FileIconFasta size={30} />, [])
  const iconGff = useMemo(() => <FileIconGff size={30} />, [])
  const iconJson = useMemo(() => <FileIconJson size={30} />, [])

  return (
    <Row noGutters>
      <Col>
        <FilePicker
          className="my-3"
          compact
          icon={iconJson}
          title={t('Reference tree')}
          exampleUrl="https://example.com/tree.json"
          pasteInstructions={t('Enter tree data in Auspice JSON v2 format')}
          input={refTree}
          error={refTreeError}
          onRemove={resetRefTree}
          onInput={setRefTree}
        />

        <FilePicker
          className="my-3"
          compact
          icon={iconFasta}
          title={t('Root sequence')}
          exampleUrl="https://example.com/root_seq.fasta"
          pasteInstructions={t('Enter root sequence data in FASTA format')}
          input={refSeq}
          error={refSeqError}
          onRemove={resetRefSeq}
          onInput={setRefSeq}
        />

        <FilePicker
          className="my-3"
          compact
          icon={iconJson}
          title={t('Quality control')}
          exampleUrl="https://example.com/qc.json"
          pasteInstructions={t('Enter QC config in JSON format')}
          input={qcConfig}
          error={qcConfigError}
          onRemove={resetQcConfig}
          onInput={setQcConfig}
        />

        <FilePicker
          className="my-3"
          compact
          icon={iconJson}
          title={t('Virus properties')}
          exampleUrl="https://example.com/virus_properties.json"
          pasteInstructions={t('Enter Virus attributes in JSON format')}
          input={virusProperties}
          error={virusPropertiesError}
          onRemove={resetVirusProperties}
          onInput={setVirusProperties}
        />

        <FilePicker
          className="my-3"
          compact
          icon={iconGff}
          title={t('Gene map')}
          exampleUrl="https://example.com/gene_map.json"
          pasteInstructions={t('Enter gene map data in JSON format')}
          input={geneMap}
          error={geneMapError}
          onRemove={resetGeneMap}
          onInput={setGeneMap}
        />

        <FilePicker
          className="my-3"
          compact
          icon={iconCsv}
          title={t('PCR primers')}
          exampleUrl="https://example.com/pcr_primers.csv"
          pasteInstructions={t('Enter PCR primers data in CSV format')}
          input={primersCsv}
          error={primersCsvError}
          onRemove={resetPrimersCsv}
          onInput={setPrimersCsv}
        />
      </Col>
    </Row>
  )
}
