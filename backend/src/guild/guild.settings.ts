import { BadRequestException } from '@nestjs/common';

export type GuildSettings = {
  visibility?: 'public' | 'private' | 'unlisted';
  requireApproval?: boolean; // joining requires approval
  discoverable?: boolean; // shows up in search
  maxMembers?: number | null;
};

export const DEFAULT_GUILD_SETTINGS: Required<GuildSettings> = {
  visibility: 'public',
  requireApproval: false,
  discoverable: true,
  maxMembers: null,
};

export function validateAndNormalizeSettings(input: any): Required<GuildSettings> {
  if (input == null) return { ...DEFAULT_GUILD_SETTINGS };
  if (typeof input !== 'object') throw new BadRequestException('Invalid settings format');

  const out: any = { ...DEFAULT_GUILD_SETTINGS };

  if ('visibility' in input) {
    if (!['public', 'private', 'unlisted'].includes(input.visibility))
      throw new BadRequestException('Invalid visibility setting');
    out.visibility = input.visibility;
  }

  if ('requireApproval' in input) {
    if (typeof input.requireApproval !== 'boolean') throw new BadRequestException('requireApproval must be boolean');
    out.requireApproval = input.requireApproval;
  }

  if ('discoverable' in input) {
    if (typeof input.discoverable !== 'boolean') throw new BadRequestException('discoverable must be boolean');
    out.discoverable = input.discoverable;
  }

  if ('maxMembers' in input) {
    if (input.maxMembers !== null && (typeof input.maxMembers !== 'number' || input.maxMembers < 1))
      throw new BadRequestException('maxMembers must be a positive number or null');
    out.maxMembers = input.maxMembers;
  }

  return out;
}
