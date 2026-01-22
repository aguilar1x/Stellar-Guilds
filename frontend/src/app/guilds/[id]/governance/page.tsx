'use client'

import React, { useState } from 'react'
import { useParams } from 'next/navigation'
import { Vote, FileText, TrendingUp, ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { ProposalStatus } from '@/types/ui'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { ProposalCard } from '@/features/governance/components/ProposalCard'
import { 
  getProposalsByGuildId, 
  getProposalsByStatus,
  mockUserVotingPower 
} from '@/lib/mocks/governance'
import { mockGuilds } from '@/lib/mocks'
import { cn } from '@/lib/utils'

export default function GovernancePage() {
  const params = useParams()
  const guildId = params.id as string
  const [statusFilter, setStatusFilter] = useState<ProposalStatus | 'all'>('all')

  const guild = mockGuilds.find(g => g.id === guildId)
  const allProposals = getProposalsByGuildId(guildId)
  const filteredProposals = statusFilter === 'all' 
    ? allProposals 
    : getProposalsByStatus(statusFilter).filter(p => p.guildId === guildId)

  const activeProposals = allProposals.filter(p => p.status === 'active')
  const totalProposals = allProposals.length

  const filterTabs: Array<{ label: string; value: ProposalStatus | 'all' }> = [
    { label: 'All', value: 'all' },
    { label: 'Active', value: 'active' },
    { label: 'Passed', value: 'passed' },
    { label: 'Rejected', value: 'rejected' }
  ]

  return (
    <div className="flex flex-col min-h-screen bg-stellar-navy">
      <div className="flex-1 p-6">
        <div className="max-w-7xl mx-auto space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div>
              <Link href={`/guilds/${guildId}`}>
                <Button variant="ghost" size="sm" leftIcon={<ArrowLeft size={16} />}>
                  Back to Guild
                </Button>
              </Link>
              <h1 className="text-3xl font-bold text-stellar-white mt-2">
                {guild?.name || 'Guild'} Governance
              </h1>
              <p className="text-stellar-slate mt-1">
                Manage proposals and participate in guild decisions
              </p>
            </div>
            <Link href={`/guilds/${guildId}/governance/create`}>
              <Button leftIcon={<FileText size={18} />}>
                Create Proposal
              </Button>
            </Link>
          </div>

          {/* Stats Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Card className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-stellar-slate mb-1">Total Proposals</p>
                  <p className="text-2xl font-bold text-stellar-white">{totalProposals}</p>
                </div>
                <div className="p-3 rounded-lg bg-stellar-navy">
                  <FileText size={24} className="text-gold-500" />
                </div>
              </div>
            </Card>

            <Card className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-stellar-slate mb-1">Active Votes</p>
                  <p className="text-2xl font-bold text-stellar-white">{activeProposals.length}</p>
                </div>
                <div className="p-3 rounded-lg bg-stellar-navy">
                  <Vote size={24} className="text-gold-500" />
                </div>
              </div>
            </Card>

            <Card className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-stellar-slate mb-1">Your Voting Power</p>
                  <p className="text-2xl font-bold text-gold-400">{mockUserVotingPower}</p>
                </div>
                <div className="p-3 rounded-lg bg-stellar-navy">
                  <TrendingUp size={24} className="text-gold-500" />
                </div>
              </div>
            </Card>
          </div>

          {/* Filter Tabs */}
          <div className="flex items-center gap-2 border-b border-stellar-lightNavy">
            {filterTabs.map((tab) => (
              <button
                key={tab.value}
                onClick={() => setStatusFilter(tab.value)}
                className={cn(
                  "px-4 py-2 text-sm font-medium transition-colors border-b-2",
                  statusFilter === tab.value
                    ? "text-gold-400 border-gold-500"
                    : "text-stellar-slate border-transparent hover:text-stellar-lightSlate"
                )}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Proposals List */}
          {filteredProposals.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {filteredProposals.map((proposal) => (
                <ProposalCard
                  key={proposal.id}
                  proposal={proposal}
                  guildId={guildId}
                />
              ))}
            </div>
          ) : (
            <Card className="p-12 text-center">
              <FileText size={48} className="mx-auto text-stellar-slate mb-4" />
              <h3 className="text-xl font-semibold text-stellar-white mb-2">
                No proposals found
              </h3>
              <p className="text-stellar-slate mb-6">
                {statusFilter === 'all' 
                  ? 'Get started by creating the first proposal for this guild.'
                  : `No ${statusFilter} proposals at this time.`
                }
              </p>
              {statusFilter === 'all' && (
                <Link href={`/guilds/${guildId}/governance/create`}>
                  <Button>Create Proposal</Button>
                </Link>
              )}
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
