use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Sources {
	Table,
	Id,
	Name,
	Version,
	Kind,
	IconUrl,
	RefererUrl,
	BaseUrl,
}

#[derive(Iden)]
pub enum Works {
	Table,
	Id,
	Kind,
	SourceId,
	RemoteUrl,
	Title,
	CoverUrl,
	AlternativeNames,
	Authors,
	Artists,
	Status,
	ReleaseDate,
	Description,
	Genres,
	CreatedAt,
	UpdatedAt,
}

#[derive(Iden)]
pub enum Chapters {
	Table,
	Id,
	WorkId,
	Title,
	RemoteUrl,
	SortIndex,
	ContentKind,
	ScanlationGroup,
	ReleasedAt,
	CreatedAt,
}

#[derive(Iden)]
pub enum Users {
	Table,
	Id,
	Username,
	PasswordHash,
	CreatedAt,
}

#[derive(Iden)]
pub enum Sessions {
	Table,
	Token,
	UserId,
	DeviceLabel,
	CreatedAt,
	LastSeenAt,
}

#[derive(Iden)]
pub enum Categories {
	Table,
	Id,
	UserId,
	Name,
	CreatedAt,
}

#[derive(Iden)]
pub enum LibraryEntries {
	Table,
	Id,
	UserId,
	WorkId,
	CategoryId,
	CreatedAt,
	UpdatedAt,
	RemovedAt,
}

#[derive(Iden)]
pub enum ReadingProgress {
	UpdatedAt,
	RemovedAt,
	Table,
	Id,
	UserId,
	WorkId,
	ChapterId,
	ReadAt,
}

#[derive(Iden)]
pub enum Jobs {
	Table,
	Id,
	Kind,
	Subject,
	Status,
	Attempts,
	NextAttemptAt,
	LastError,
	CreatedAt,
	UpdatedAt,
}

#[derive(Iden)]
pub enum ServerSettings {
	Table,
	Key,
	Value,
}

#[derive(Iden)]
pub enum InviteCodes {
	Table,
	Id,
	Code,
	CreatedBy,
	UsedBy,
	CreatedAt,
	UsedAt,
}

#[derive(Iden)]
pub enum UserSettings {
	Table,
	UserId,
	ApiKeyEnc,
	ProviderBaseUrl,
	ProviderModel,
}

#[derive(Iden)]
pub enum TranslationCache {
	Table,
	Key,
	Content,
	CreatedAt,
}

#[derive(Iden)]
pub enum GlossaryEntries {
	Table,
	Id,
	Term,
	Language,
	Romanization,
	CreatedAt,
}

#[derive(Iden)]
pub enum GlossaryMeanings {
	Table,
	Id,
	EntryId,
	Meaning,
	Votes,
	CreatedBy,
	CreatedAt,
}

#[derive(Iden)]
pub enum GlossaryVotes {
	Table,
	UserId,
	MeaningId,
}

#[derive(Iden)]
pub enum TrackerAccounts {
	Table,
	UserId,
	TrackerId,
	AccessTokenEnc,
	RefreshTokenEnc,
	AccountLabel,
	UpdatedAt,
}

#[derive(Iden)]
pub enum TrackerLinks {
	Table,
	Id,
	UserId,
	WorkId,
	TrackerId,
	RemoteId,
	RemoteTitle,
	RemoteStatus,
	Score,
	LastChaptersSynced,
	UpdatedAt,
}

#[derive(Iden)]
pub enum ReadingPositions {
	Table,
	Id,
	UserId,
	ChapterId,
	Page,
	UpdatedAt,
	RemovedAt,
}
