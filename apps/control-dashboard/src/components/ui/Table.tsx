import type { ReactNode } from 'react'

interface Column<T> {
  key: string
  header: string
  render?: (row: T) => ReactNode
}

interface MobileCardConfig {
  /** Column key for the card header (primary field) */
  headerColumn: string
  /** Column key for an inline badge in the card header */
  badgeColumn?: string
  /** Column keys to display as label:value pairs in card body */
  bodyColumns: string[]
  /** Column keys to hide entirely on mobile */
  hiddenColumns?: string[]
}

interface TableProps<T> {
  columns: Column<T>[]
  data: T[]
  onRowClick?: (row: T) => void
  mobileCardConfig?: MobileCardConfig
}

function renderCellValue<T extends object>(row: T, col: Column<T>): ReactNode {
  if (col.render) return col.render(row)
  return String((row as Record<string, unknown>)[col.key] ?? '')
}

function CardList<T extends object>({
  columns,
  data,
  onRowClick,
  config,
}: {
  columns: Column<T>[]
  data: T[]
  onRowClick?: (row: T) => void
  config: MobileCardConfig
}) {
  const colMap = new Map(columns.map((c) => [c.key, c]))
  const headerCol = colMap.get(config.headerColumn)
  const badgeCol = config.badgeColumn ? colMap.get(config.badgeColumn) : undefined
  const bodyCols = config.bodyColumns
    .map((key) => colMap.get(key))
    .filter((c): c is Column<T> => c !== undefined)

  return (
    <div className="space-y-3 sm:hidden">
      {data.map((row, i) => (
        <div
          key={i}
          onClick={onRowClick ? () => onRowClick(row) : undefined}
          className={`rounded-lg border border-secondary-200 bg-white p-4 ${
            onRowClick ? 'cursor-pointer active:bg-secondary-50' : ''
          }`}
        >
          <div className="flex items-center justify-between gap-2">
            <span className="font-medium text-secondary-900 truncate">
              {headerCol ? renderCellValue(row, headerCol) : ''}
            </span>
            {badgeCol && (
              <span className="shrink-0">{renderCellValue(row, badgeCol)}</span>
            )}
          </div>
          {bodyCols.length > 0 && (
            <div className="mt-2 space-y-1">
              {bodyCols.map((col) => (
                <div key={col.key} className="flex items-baseline justify-between gap-2 text-sm">
                  <span className="text-secondary-500 shrink-0">{col.header}</span>
                  <span className="text-secondary-700 text-right truncate">
                    {renderCellValue(row, col)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

export function Table<T extends object>({ columns, data, onRowClick, mobileCardConfig }: TableProps<T>) {
  return (
    <>
      {mobileCardConfig && (
        <CardList columns={columns} data={data} onRowClick={onRowClick} config={mobileCardConfig} />
      )}
      <div className={`overflow-x-auto ${mobileCardConfig ? 'hidden sm:block' : ''}`}>
        <table className="min-w-full divide-y divide-secondary-200">
          <thead className="bg-secondary-50">
            <tr>
              {columns.map((col) => (
                <th
                  key={col.key}
                  className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-secondary-500"
                >
                  {col.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-secondary-200 bg-white">
            {data.map((row, i) => (
              <tr
                key={i}
                onClick={onRowClick ? () => onRowClick(row) : undefined}
                className={`text-sm text-secondary-900 ${onRowClick ? 'cursor-pointer hover:bg-secondary-50' : ''}`}
              >
                {columns.map((col) => (
                  <td key={col.key} className="whitespace-nowrap px-4 py-3">
                    {renderCellValue(row, col)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  )
}
