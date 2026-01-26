export type BountyStatus = 'Open' | 'Claimed' | 'Under Review' | 'Completed';
export type Difficulty = 'Beginner' | 'Intermediate' | 'Advanced';

export interface Bounty {
  id: string;
  title: string;
  description: string;
  rewardAmount: number;
  tokenSymbol: string;
  tokenIcon: string;
  difficulty: Difficulty;
  status: BountyStatus;
  guildName: string;
  guildLogo: string;
  deadline: string;
  applicants: number;
  tags: string[];
}