import React, { memo, useMemo } from 'react'
import { areEqual, ListChildComponentProps } from 'react-window'
import { useRecoilValue } from 'recoil'

import type { CladeNodeAttrDesc } from 'auspice'
import type { PhenotypeAttrDesc } from 'src/types'
import { COLUMN_WIDTHS } from 'src/components/Results/ResultsTableStyle'
import { analysisResultAtom } from 'src/state/results.state'
import { ResultsTableRowError } from './ResultsTableRowError'
import { ResultsTableRowPending } from './ResultsTableRowPending'
import { ResultsTableRowResult } from './ResultsTableRowResult'

export interface TableRowDatum {
  seqIndex: number
  viewedGene: string
  columnWidthsPx: Record<keyof typeof COLUMN_WIDTHS, string>
  dynamicCladeColumnWidthPx: string
  dynamicPhenotypeColumnWidthPx: string
  cladeNodeAttrDescs: CladeNodeAttrDesc[]
  phenotypeAttrDescs: PhenotypeAttrDesc[]
}

export interface RowProps extends ListChildComponentProps {
  data: TableRowDatum[]
}

export const ResultsTableRow = memo(ResultsTableRowUnmemoed, areEqual)

function ResultsTableRowUnmemoed({ index, data, ...restProps }: RowProps) {
  const {
    seqIndex,
    viewedGene,
    columnWidthsPx,
    dynamicCladeColumnWidthPx,
    dynamicPhenotypeColumnWidthPx,
    cladeNodeAttrDescs,
    phenotypeAttrDescs,
  } = useMemo(() => data[index], [data, index])

  const { result, error } = useRecoilValue(analysisResultAtom(seqIndex))

  if (error) {
    return <ResultsTableRowError {...restProps} index={seqIndex} columnWidthsPx={columnWidthsPx} />
  }

  if (result) {
    return (
      <ResultsTableRowResult
        {...restProps}
        index={seqIndex}
        columnWidthsPx={columnWidthsPx}
        dynamicCladeColumnWidthPx={dynamicCladeColumnWidthPx}
        dynamicPhenotypeColumnWidthPx={dynamicPhenotypeColumnWidthPx}
        cladeNodeAttrDescs={cladeNodeAttrDescs}
        phenotypeAttrDescs={phenotypeAttrDescs}
        viewedGene={viewedGene}
      />
    )
  }

  return <ResultsTableRowPending index={seqIndex} columnWidthsPx={columnWidthsPx} />
}
