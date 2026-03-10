import type { AppVersion } from '../api/types';

interface PlatformBadgeProps {
  platform: AppVersion['platform'];
}

const platformLabels: Record<AppVersion['platform'], string> = {
  windows: 'Windows',
  linux: 'Linux',
  macos: 'macOS',
};

const platformIcons: Record<AppVersion['platform'], string> = {
  windows: '\u{1F5A5}',
  linux: '\u{1F427}',
  macos: '\u{1F34E}',
};

export function PlatformBadge({ platform }: PlatformBadgeProps) {
  return (
    <span className="platform-badge" data-platform={platform}>
      <span className="platform-badge__icon">{platformIcons[platform]}</span>
      <span className="platform-badge__label">{platformLabels[platform]}</span>
    </span>
  );
}
