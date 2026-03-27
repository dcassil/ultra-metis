import { Badge } from './ui/Badge'

const tierConfig: Record<string, { variant: 'info' | 'pending' | 'offline'; label: string }> = {
  trusted: { variant: 'info', label: 'Trusted' },
  restricted: { variant: 'pending', label: 'Restricted' },
}

export function TrustTierBadge({ tier }: { tier: string }) {
  const config = tierConfig[tier]
  if (config) {
    return <Badge variant={config.variant}>{config.label}</Badge>
  }
  return <Badge variant="offline">{tier}</Badge>
}
