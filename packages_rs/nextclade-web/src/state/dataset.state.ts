import { isNil } from 'lodash'
import { atom, DefaultValue, selector } from 'recoil'

import type { Dataset, MinimizerIndexVersion } from 'src/types'
// import { GENE_OPTION_NUC_SEQUENCE } from 'src/constants'
import { persistAtom } from 'src/state/persist/localStorage'
// import { viewedGeneAtom } from 'src/state/seqViewSettings.state'
import { isDefaultValue } from 'src/state/utils/isDefaultValue'
import { areDatasetsEqual } from 'src/types'

export interface Datasets {
  datasets: Dataset[]
}

export const datasetServerUrlAtom = atom<string>({
  key: 'datasetServerUrlAtom',
  default: '/',
})

export const datasetsAtom = atom<Datasets>({
  key: 'datasets',
})

const datasetCurrentStorageAtom = atom<Dataset | undefined>({
  key: 'datasetCurrentStorage',
  default: undefined,
  effects: [persistAtom],
})

export const datasetCurrentAtom = selector<Dataset | undefined>({
  key: 'datasetCurrent',
  get({ get }) {
    return get(datasetCurrentStorageAtom)
  },
  set({ get, set, reset }, dataset: Dataset | undefined | DefaultValue) {
    const datasetCurrent = get(datasetCurrentStorageAtom)
    if (isDefaultValue(dataset) || isNil(dataset)) {
      reset(datasetCurrentStorageAtom)
    } else if (!areDatasetsEqual(datasetCurrent, dataset)) {
      set(datasetCurrentStorageAtom, dataset)
    }
  },
})

export const datasetUpdatedAtom = atom<Dataset | undefined>({
  key: 'datasetUpdated',
  default: undefined,
})

export const geneOrderPreferenceAtom = atom<string[]>({
  key: 'geneOrderPreference',
  default: [],
})

export const minimizerIndexVersionAtom = atom<MinimizerIndexVersion | undefined>({
  key: 'minimizerIndexVersionAtom',
  default: undefined,
})
