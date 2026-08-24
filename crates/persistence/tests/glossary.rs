use persistence::{GlossaryRepository, SeaStore, UserRepository};

async fn store() -> SeaStore {
	let db = persistence::connect("sqlite::memory:").await.unwrap();
	SeaStore::new(db)
}

#[tokio::test]
async fn glossary_create_vote_roundtrip() {
	let store = store().await;
	store.create_user("alice", "").await.unwrap();
	let user = store.get_user_by_username("alice").await.unwrap().unwrap().id;

	store
		.create_glossary_entry("修羅場", "ja", Some("shuraba"), "love triangle meltdown", user)
		.await
		.unwrap();
	assert!(
		store
			.create_glossary_entry("修羅場", "ja", None, "duplicate", user)
			.await
			.is_err()
	);

	let entries = store.glossary_for_language("ja", user).await.unwrap();
	assert_eq!(entries.len(), 1);
	assert_eq!(entries[0].meanings.len(), 1);
	assert!(entries[0].meanings[0].voted_by_me);

	let meaning_id = entries[0].meanings[0].id;
	let alternative = store.add_glossary_meaning(entries[0].id, "drama chaos", user).await.unwrap();

	assert!(store.toggle_glossary_vote(user, alternative.id).await.unwrap());
	assert!(
		!store.toggle_glossary_vote(user, alternative.id).await.unwrap(),
		"second toggle removes the vote"
	);

	let entries = store.glossary_for_language("ja", user).await.unwrap();
	let top = entries[0].top_meaning().unwrap();
	assert_eq!(top.meaning, "love triangle meltdown", "original meaning keeps the lead");
	assert!(!alternative.voted_by_me || top.id == alternative.id);

	let _ = meaning_id;
}

#[tokio::test]
async fn votes_rank_meanings() {
	let store = store().await;
	store.create_user("alice", "").await.unwrap();
	store.create_user("bob", "").await.unwrap();
	let alice = store.get_user_by_username("alice").await.unwrap().unwrap().id;
	let bob = store.get_user_by_username("bob").await.unwrap().unwrap().id;

	let entry = store
		.create_glossary_entry("tsundere", "ja", None, "cold outside", alice)
		.await
		.unwrap();
	let warmer = store.add_glossary_meaning(entry.id, "warm inside", bob).await.unwrap();
	store.toggle_glossary_vote(alice, warmer.id).await.unwrap();
	store.toggle_glossary_vote(bob, warmer.id).await.unwrap();

	let entries = store.glossary_for_language("ja", alice).await.unwrap();
	assert_eq!(entries[0].meanings[0].meaning, "warm inside");
	assert!(entries[0].meanings[0].votes >= 2);
}
