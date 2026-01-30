import { Injectable, NotFoundException, ForbiddenException, BadRequestException, ConflictException, InternalServerErrorException } from '@nestjs/common';
import { validateAndNormalizeSettings } from './guild.settings';
import { PrismaService } from '../prisma/prisma.service';
import { CreateGuildDto } from './dto/create-guild.dto';
import { UpdateGuildDto } from './dto/update-guild.dto';
import { InviteMemberDto } from './dto/invite-member.dto';
import { randomUUID } from 'crypto';

@Injectable()
export class GuildService {
  constructor(private prisma: PrismaService) {}

  private slugify(name: string) {
    return name
      .toLowerCase()
      .replace(/\s+/g, '-')
      .replace(/[^a-z0-9-]/g, '')
      .substring(0, 100);
  }

  async createGuild(dto: CreateGuildDto, ownerId: string) {
    const slug = dto.slug ? dto.slug : this.slugify(dto.name);

    // Pre-check slug uniqueness to provide friendlier error
    const existing = await this.prisma.guild.findUnique({ where: { slug } });
    if (existing) throw new ConflictException('Slug already in use');

    const normalizedSettings = validateAndNormalizeSettings((dto as any).settings);

    let guild;
    try {
      guild = await this.prisma.guild.create({
        data: {
          name: dto.name,
          slug,
          description: dto.description,
          ownerId,
          settings: normalizedSettings,
        },
      });
    } catch (err: any) {
      // Handle Prisma unique constraint race or other DB errors
      if (err.code === 'P2002') {
        throw new ConflictException('Guild with that slug or unique field already exists');
      }
      throw new InternalServerErrorException('Failed to create guild');
    }

    // create owner membership
    await this.prisma.guildMembership.create({
      data: {
        userId: ownerId,
        guildId: guild.id,
        role: 'OWNER',
        status: 'APPROVED',
        joinedAt: new Date(),
      },
    });

    return guild;
  }

  async getGuild(id: string) {
    const guild = await this.prisma.guild.findUnique({
      where: { id },
      include: { memberships: { include: { user: true } } },
    });
    if (!guild) throw new NotFoundException('Guild not found');
    return guild;
  }

  async getBySlug(slug: string) {
    const guild = await this.prisma.guild.findUnique({
      where: { slug },
      include: { memberships: { include: { user: true } } },
    });
    if (!guild) throw new NotFoundException('Guild not found');
    return guild;
  }

  private async ensureManagePermission(guildId: string, userId: string) {
    const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
    if (!guild) throw new NotFoundException('Guild not found');
    if (guild.ownerId === userId) return;

    const membership = await this.prisma.guildMembership.findUnique({
      where: { userId_guildId: { userId, guildId } },
    });
    if (!membership) throw new ForbiddenException('Not a member');
    if (membership.role === 'MEMBER') throw new ForbiddenException('Insufficient guild permissions');
  }

  async updateGuild(guildId: string, dto: UpdateGuildDto, userId: string) {
    await this.ensureManagePermission(guildId, userId);
    // If settings present, validate and merge with existing
    const data: any = { ...dto };
    if ((dto as any).settings) {
      const existing = await this.prisma.guild.findUnique({ where: { id: guildId } });
      const validated = validateAndNormalizeSettings((dto as any).settings);
      data.settings = { ...existing.settings, ...validated };
    }

    return this.prisma.guild.update({ where: { id: guildId }, data });
  }

  async deleteGuild(guildId: string, userId: string) {
    const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
    if (!guild) throw new NotFoundException('Guild not found');
    if (guild.ownerId !== userId) throw new ForbiddenException('Only owner can delete the guild');
    return this.prisma.guild.delete({ where: { id: guildId } });
  }

  async searchGuilds(q: string | undefined, page = 0, size = 20) {
    const where = q
      ? {
          OR: [
            { name: { contains: q, mode: 'insensitive' } },
            { description: { contains: q, mode: 'insensitive' } },
          ],
        }
      : {};

    const [items, total] = await Promise.all([
      this.prisma.guild.findMany({ where, skip: page * size, take: size }),
      this.prisma.guild.count({ where }),
    ]);

    return { items, total, page, size };
  }

  async inviteMember(guildId: string, dto: InviteMemberDto, invitedBy: string) {
    await this.ensureManagePermission(guildId, invitedBy);

    const existing = await this.prisma.guildMembership.findUnique({
      where: { userId_guildId: { userId: dto.userId, guildId } },
    });
    if (existing) throw new BadRequestException('User already invited or member');

    const token = randomUUID();

    const membership = await this.prisma.guildMembership.create({
      data: {
        userId: dto.userId,
        guildId,
        role: (dto.role as any) || 'MEMBER',
        status: 'PENDING',
        invitationToken: token,
        invitedById: invitedBy,
      },
    });

    return { membership, token };
  }

  async approveInviteByToken(guildId: string, token: string, approverId: string) {
    const membership = await this.prisma.guildMembership.findFirst({ where: { guildId, invitationToken: token } });
    if (!membership) throw new NotFoundException('Invite not found');

    // If approver is the invitee, allow; otherwise check permission
    if (membership.userId !== approverId) await this.ensureManagePermission(guildId, approverId);

    const updated = await this.prisma.guildMembership.update({
      where: { id: membership.id },
      data: { status: 'APPROVED', joinedAt: new Date(), invitationToken: null },
    });

    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });

    return updated;
  }

  async approveInviteForUser(guildId: string, userId: string) {
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (!membership) throw new NotFoundException('Invite not found');
    if (membership.status !== 'PENDING') throw new BadRequestException('No pending invite to approve');

    const updated = await this.prisma.guildMembership.update({ where: { id: membership.id }, data: { status: 'APPROVED', joinedAt: new Date(), invitationToken: null } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
    return updated;
  }

  async joinGuild(guildId: string, userId: string) {
    const existing = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (existing && existing.status === 'APPROVED') return existing;
    if (existing && existing.status === 'PENDING') {
      const updated = await this.prisma.guildMembership.update({ where: { id: existing.id }, data: { status: 'APPROVED', joinedAt: new Date() } });
      await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
      return updated;
    }

    const created = await this.prisma.guildMembership.create({ data: { userId, guildId, role: 'MEMBER', status: 'APPROVED', joinedAt: new Date() } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
    return created;
  }

  async leaveGuild(guildId: string, userId: string) {
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (!membership) throw new NotFoundException('Not a member');
    if (membership.role === 'OWNER') throw new BadRequestException('Owner cannot leave the guild');
    await this.prisma.guildMembership.delete({ where: { id: membership.id } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { decrement: 1 } as any } as any });
    return { success: true };
  }

  async assignRole(guildId: string, targetUserId: string, role: string, byUserId: string) {
    await this.ensureManagePermission(guildId, byUserId);
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId: targetUserId, guildId } } });
    if (!membership) throw new NotFoundException('Member not found');
    return this.prisma.guildMembership.update({ where: { id: membership.id }, data: { role: role as any } });
  }
}
