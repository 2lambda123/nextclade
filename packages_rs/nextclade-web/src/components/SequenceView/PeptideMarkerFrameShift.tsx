import React, { SVGProps, useState } from 'react'
import { useRecoilValue } from 'recoil'

import { BASE_MIN_WIDTH_PX } from 'src/constants'
import type { FrameShift } from 'src/algorithms/types'
import { useTranslationSafe } from 'src/helpers/useTranslationSafe'
import { Tooltip } from 'src/components/Results/Tooltip'
import { TableRowSpacer, TableSlim } from 'src/components/Common/TableSlim'
import { formatRange, formatRangeMaybeEmpty } from 'src/helpers/formatRange'
import { getSafeId } from 'src/helpers/getSafeId'
import { geneMapAtom } from 'src/state/results.state'

const frameShiftColor = '#eb0d2a'
const frameShiftBorderColor = '#ffff00'

export interface PeptideMarkerFrameShiftProps extends SVGProps<SVGRectElement> {
  seqName: string
  frameShift: FrameShift
  pixelsPerAa: number
}

function PeptideMarkerFrameShiftUnmemoed({ seqName, frameShift, pixelsPerAa, ...rest }: PeptideMarkerFrameShiftProps) {
  const { t } = useTranslationSafe()
  const [showTooltip, setShowTooltip] = useState(false)

  const { geneName, nucAbs, codon, gapsLeading, gapsTrailing } = frameShift
  const id = getSafeId('frame-shift-aa-marker', { seqName, ...frameShift })

  const geneMap = useRecoilValue(geneMapAtom)

  const gene = geneMap.find((gene) => geneName === gene.geneName)
  if (!gene) {
    return null
  }

  const nucLength = nucAbs.end - nucAbs.begin
  const codonLength = codon.end - codon.begin

  let width = codonLength * pixelsPerAa
  width = Math.max(width, BASE_MIN_WIDTH_PX)
  const halfAa = Math.max(pixelsPerAa, BASE_MIN_WIDTH_PX) / 2 // Anchor on the center of the first AA
  const x = codon.begin * pixelsPerAa - halfAa

  const codonRangeStr = formatRange(codon.begin, codon.end)
  const nucRangeStr = formatRange(nucAbs.begin, nucAbs.end)

  return (
    <g id={id}>
      <rect
        fill={frameShiftBorderColor}
        x={x - 1}
        y={1.75}
        width={width + 2}
        stroke={frameShiftBorderColor}
        strokeWidth={0.5}
        height={7}
      />
      <rect
        id={id}
        fill={frameShiftColor}
        x={x}
        y={2.5}
        width={width}
        height="5"
        {...rest}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
      >
        <Tooltip target={id} isOpen={showTooltip} fullWidth>
          <h5>{t('Frame shift')}</h5>

          <TableSlim borderless className="mb-1">
            <thead />
            <tbody>
              <tr>
                <td>{t('Nucleotide range')}</td>
                <td>{nucRangeStr}</td>
              </tr>

              <tr>
                <td>{t('Nucleotide length')}</td>
                <td>{nucLength}</td>
              </tr>

              <tr>
                <td>{t('Gene')}</td>
                <td>{geneName}</td>
              </tr>

              <tr>
                <td>{t('Codon range')}</td>
                <td>{codonRangeStr}</td>
              </tr>

              <tr>
                <td>{t('Codon length')}</td>
                <td>{codonLength}</td>
              </tr>

              <TableRowSpacer />

              <tr>
                <td>{t('Leading deleted codon range')}</td>
                <td>{formatRangeMaybeEmpty(gapsLeading.codon.begin, gapsLeading.codon.end)}</td>
              </tr>

              <tr>
                <td>{t('Trailing deleted codon range')}</td>
                <td>{formatRangeMaybeEmpty(gapsTrailing.codon.begin, gapsTrailing.codon.end)}</td>
              </tr>
            </tbody>
          </TableSlim>
        </Tooltip>
      </rect>
    </g>
  )
}

export const PeptideMarkerFrameShift = React.memo(PeptideMarkerFrameShiftUnmemoed)
