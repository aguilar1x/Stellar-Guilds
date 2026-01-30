import { IsString, IsOptional, MaxLength, IsObject } from 'class-validator';

export class CreateGuildDto {
  @IsString()
  @MaxLength(100)
  name: string;

  @IsOptional()
  @IsString()
  @MaxLength(100)
  slug?: string;

  @IsOptional()
  @IsString()
  @MaxLength(1000)
  description?: string;

  @IsOptional()
  @IsObject()
  settings?: any;
}
