import type { AuspiceJsonV2, CladeNodeAttrDesc } from 'auspice'
import { isNil } from 'lodash'
import { atom, atomFamily, DefaultValue, selector, selectorFamily } from 'recoil'
import { AlgorithmGlobalStatus, AlgorithmSequenceStatus, getResultStatus } from 'src/algorithms/types'

import type { Gene, NextcladeResult } from 'src/algorithms/types'
import { runFilters } from 'src/filtering/runFilters'
import { SortCategory, SortDirection, sortResults, sortResultsByKey } from 'src/helpers/sortResults'
import { analysisStatusGlobalAtom } from 'src/state/analysisStatusGlobal.state'
import {
  aaFilterAtom,
  cladesFilterAtom,
  mutationsFilterAtom,
  seqNamesFilterAtom,
  showBadFilterAtom,
  showErrorsFilterAtom,
  showGoodFilterAtom,
  showMediocreFilterAtom,
} from 'src/state/resultFilters.state'

export function isDefaultValue(candidate: unknown): candidate is DefaultValue {
  return candidate instanceof DefaultValue
}

// Stores analysis result for a single sequence (defined by sequence name)
// Do not use setState on this atom directly, use `analysisResultAtom` instead!
const analysisResultInternalAtom = atomFamily<NextcladeResult, number>({
  key: 'analysisResultSingle',
})

// Stores sequence names as they come from fasta
// Do not use setState on this atom directly, use `analysisResultAtom` instead!
export const seqIndicesAtom = atom<number[]>({
  key: 'seqIndices',
  default: [],
})

// Stores a map from sequence index to an array od sequences with the same name
export const seqNameDuplicatesAtom = atomFamily<number[], string>({
  key: 'seqNameDuplicates',
  default: [],
})

// Synchronizes states of `analysisResultAtom` and `seqIndicesAtom`
// Use it to set `analysisResultInternalAtom` and `seqIndicesAtom`
export const analysisResultAtom = selectorFamily<NextcladeResult, number>({
  key: 'analysisResult',

  get:
    (index: number) =>
    ({ get }): NextcladeResult => {
      return get(analysisResultInternalAtom(index))
    },

  set:
    (index) =>
    ({ get, set, reset }, result: NextcladeResult | DefaultValue) => {
      if (isDefaultValue(result)) {
        const result = get(analysisResultInternalAtom(index))
        reset(seqNameDuplicatesAtom(result.seqName))
        reset(analysisResultInternalAtom(index))
        reset(seqIndicesAtom)
      } else {
        set(analysisResultInternalAtom(index), result)

        // Add to the list of indices
        set(seqIndicesAtom, (prev) => {
          if (result && !prev.includes(result.index)) {
            return [...prev, result.index]
          }
          return prev
        })

        // Add to the duplicate names map
        const indices = get(seqNameDuplicatesAtom(result.seqName))
        set(seqNameDuplicatesAtom(result.seqName), [...indices, result.index])
      }
    },
})

export const seqIndicesFilteredAtom = selector<number[]>({
  key: 'seqIndicesFiltered',

  get: ({ get }) => {
    const results = get(analysisResultsAtom)

    const filters = {
      seqNamesFilter: get(seqNamesFilterAtom),
      mutationsFilter: get(mutationsFilterAtom),
      aaFilter: get(aaFilterAtom),
      cladesFilter: get(cladesFilterAtom),
      showGood: get(showGoodFilterAtom),
      showMediocre: get(showMediocreFilterAtom),
      showBad: get(showBadFilterAtom),
      showErrors: get(showErrorsFilterAtom),
    }

    const resultsFiltered = runFilters(results, filters)

    return resultsFiltered.map(({ index }) => index)
  },
})

export const sortAnalysisResultsAtom = selectorFamily<undefined, { category: SortCategory; direction: SortDirection }>({
  key: 'sortAnalysisResults',

  get: () => () => undefined,

  set:
    ({ category, direction }) =>
    ({ get, set }, def: undefined | DefaultValue) => {
      const results = get(analysisResultsAtom)

      let sortCategory = category
      if (isDefaultValue(def)) {
        sortCategory = SortCategory.index
      }

      const resultsSorted = sortResults(results, { category: sortCategory, direction })
      const seqIndicesSorted = resultsSorted.map((result) => result.index)

      set(seqIndicesAtom, seqIndicesSorted)
    },
})

export const sortAnalysisResultsByKeyAtom = selectorFamily<undefined, { key: string; direction: SortDirection }>({
  key: 'sortAnalysisResultsByKey',

  get: () => () => undefined,

  set:
    ({ key, direction }) =>
    ({ get, set }, def: undefined | DefaultValue) => {
      const results = get(analysisResultsAtom)

      const resultsSorted = isDefaultValue(def)
        ? sortResults(results, { category: SortCategory.index, direction })
        : sortResultsByKey(results, { key, direction })

      const seqIndicesSorted = resultsSorted.map((result) => result.index)
      set(seqIndicesAtom, seqIndicesSorted)
    },
})

/**
 * Access array of analysis results
 * NOTE: `set` operation will replace the existing elements in the array with the new ones
 */
export const analysisResultsAtom = selector<NextcladeResult[]>({
  key: 'analysisResults',

  get({ get }): NextcladeResult[] {
    const seqIndices = get(seqIndicesAtom)
    return seqIndices.map((index) => get(analysisResultAtom(index)))
  },

  set({ get, set, reset }, results: NextcladeResult[] | DefaultValue) {
    const seqIndices = get(seqIndicesAtom)

    // Remove all results
    seqIndices.forEach((index) => {
      reset(analysisResultAtom(index))
    })

    // If the operation is not 'reset', add the new items
    if (!isDefaultValue(results)) {
      results.forEach((result) => set(analysisResultAtom(result.index), result))
    }
  },
})

// Selects an array of statues of all results
export const analysisResultStatusesAtom = selector<AlgorithmSequenceStatus[]>({
  key: 'analysisResultStatuses',
  get: ({ get }) => {
    const seqIndices = get(seqIndicesAtom)
    return seqIndices.map((index) => {
      const result = get(analysisResultInternalAtom(index))
      return getResultStatus(result)
    })
  },
})

export const genomeSizeAtom = atom<number>({
  key: 'genomeSize',
})

export const geneMapAtom = atom<Gene[]>({
  key: 'geneMap',
  default: [],
})

export const geneNamesAtom = selector<string[]>({
  key: 'geneNames',
  get: ({ get }) => get(geneMapAtom).map((gene) => gene.geneName),
})

export const geneAtom = selectorFamily<Gene | undefined, string>({
  key: 'gene',
  get:
    (geneName) =>
    ({ get }) => {
      const geneMap = get(geneMapAtom)
      return geneMap.find((gene) => gene.geneName === geneName)
    },
})

export const treeAtom = atom<AuspiceJsonV2 | undefined>({
  key: 'tree',
  default: undefined,
})

export const hasTreeAtom = selector<boolean>({
  key: 'hasTree',
  get({ get }) {
    return !isNil(get(treeAtom))
  },
})

export const cladeNodeAttrDescsAtom = atom<CladeNodeAttrDesc[]>({
  key: 'cladeNodeAttrDescs',
  default: [],
})

export const cladeNodeAttrKeysAtom = selector<string[]>({
  key: 'cladeNodeAttrKeys',
  get: ({ get }) => get(cladeNodeAttrDescsAtom).map((desc) => desc.name),
})

export const canDownloadAtom = selector<boolean>({
  key: 'canDownload',
  get({ get }) {
    const globalStatus = get(analysisStatusGlobalAtom)
    return globalStatus === AlgorithmGlobalStatus.done
  },
})
