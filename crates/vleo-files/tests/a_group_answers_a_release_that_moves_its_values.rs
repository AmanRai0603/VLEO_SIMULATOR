//! A group answers another group's release that moves its values: *fine*,
//! or *object, because…*, signed by its owner (docs/briefs/F5.md, item 4;
//! W7; docs/OPERATING_1_0.md, section 10, the preview answer).
//!
//! A small programme of invented people: Pat the programme manager, Sam the
//! system engineer, Ola who owns solar, Pia who owns power and Dee her
//! deputy, Nia a node engineer. Solar's release 1.2 moves power's values,
//! and power answers it.
//!
//! - Pia answers fine, or objects saying why; Dee, her deputy, answers too;
//!   each checks through the chain as power's owner's answer;
//! - refused, by name: Ola answering for power, Nia, a group answering its
//!   own release, an objection that says nothing, an assistant's name, a
//!   version that is not one;
//! - an answer changed after it was signed does not check;
//! - in the copy, written into integration/, and never written over.

use vleo_files::answer::{self, Said};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, k: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(k, name),
    });
}

fn mount(f: &mut File, uid: &str, group: &str, owner: &str) {
    f.blocks.push(Block {
        uid: s(uid),
        id: s(uid),
        behaviour: s("children"),
        ..Block::default()
    });
    f.mounts.push(Mount {
        block_uid: s(uid),
        group_id: s(group),
        release: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s(uid),
        person: s(owner),
        contract_version: 1,
    });
}

fn group_file(id: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s("whoever"));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

// Pat (11), Sam (12), Ola who owns solar (13), Pia who owns power (15),
// Dee her deputy (14), Nia a node engineer (16).
fn programme() -> Registry {
    let mut f = group_file("programme");
    register(&mut f, "Pat Morgan", "programme manager", "", &key(11));
    register(&mut f, "Sam Engel", "system engineer", "", &key(12));
    register(&mut f, "Ola Sun", "subsystem engineer", "", &key(13));
    register(&mut f, "Pia Power", "subsystem engineer", "", &key(15));
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Pia Power",
        &key(14),
    );
    register(&mut f, "Nia Node", "node engineer", "", &key(16));
    mount(&mut f, "root", "systems", "Sam Engel");
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

fn systems(pr: &Registry) -> Registry {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    mount(&mut f, "sys_power", "l3_power", "Pia Power");
    chain::open_systems(&signed(f, "Sam Engel", &key(12)), pr).unwrap()
}

/// `group`'s release `version`, as an answer reads it: named by the
/// fingerprint of what it holds.
fn release_of(group: &str, version: &str) -> File {
    let mut f = File::new(Kind::GroupRelease, APP);
    f.meta.insert(s("group_id"), s(group));
    f.meta.insert(s("version"), s(version));
    f.texts.push(Text {
        scope: s(FILE),
        kind: s("note"),
        body: format!("{group} {version}"),
    });
    let fingerprint = chain::digest(&f, FILE).unwrap();
    f.meta.insert(s("fingerprint"), fingerprint);
    f
}

fn answered(said: &Said, who: &str, k: u8) -> Result<File, String> {
    answered_to("l3_solar", "1.2", "l3_power", said, who, k)
}

fn answered_to(
    group: &str,
    version: &str,
    answering: &str,
    said: &Said,
    who: &str,
    k: u8,
) -> Result<File, String> {
    let pr = programme();
    let sy = systems(&pr);
    answer::answer(
        &release_of(group, version),
        answering,
        said,
        who,
        &key(k),
        AT,
        APP,
        &sy,
        &pr,
    )
    .map_err(|e| e.message().to_string())
}

#[test]
fn a_group_s_owner_answers_fine_or_objects_saying_why_signed() {
    let pr = programme();
    let sy = systems(&pr);
    let fine = answered(&Said::Fine, "Pia Power", 15).unwrap();
    assert_eq!(fine.kind().unwrap(), Kind::PreviewAnswer);
    let answers = chain::digest(&release_of("l3_solar", "1.2"), FILE).unwrap();
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("version", "1.2"),
        ("answers", answers.as_str()),
        ("answering_group", "l3_power"),
        ("writer", "Pia Power"),
    ] {
        assert_eq!(fine.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    assert_eq!(answer::said(&fine).unwrap(), Said::Fine);
    chain::check_answer(&fine, &sy, &pr).unwrap();
    let because = "the array's margin at end of life falls below what power holds";
    let object = answered(&Said::Object(s(because)), "Pia Power", 15).unwrap();
    assert_eq!(answer::said(&object).unwrap(), Said::Object(s(because)));
    chain::check_answer(&object, &sy, &pr).unwrap();
    // Her deputy answers for her.
    let dee = answered(&Said::Fine, "Dee Puty", 14).unwrap();
    chain::check_answer(&dee, &sy, &pr).unwrap();

    // Signed by Ola, who does not own power, past every refusal of the
    // library's own: the chain does not count it.
    let mut forged = fine.clone();
    forged.signatures.clear();
    forged.meta.insert(s("writer"), s("Ola Sun"));
    let row = chain::sign(&forged, FILE, "Ola Sun", &key(13), AT, "ok", "").unwrap();
    forged.signatures.push(row);
    let e = chain::check_answer(&forged, &sy, &pr).unwrap_err();
    assert!(
        e.message().contains("Ola Sun may not sign"),
        "{}",
        e.message()
    );
    // Changed after it was signed: it does not check.
    let mut changed = object.clone();
    for t in changed.texts.iter_mut().filter(|t| t.kind == "because") {
        t.body = s("nothing at all");
    }
    assert!(chain::check_answer(&changed, &sy, &pr).is_err());

    for (said, who, k, says) in [
        (Said::Fine, "Ola Sun", 13, "may not answer for l3_power"),
        (Said::Fine, "Nia Node", 16, "may not answer for l3_power"),
        (Said::Object(s("  ")), "Pia Power", 15, "says why"),
        (Said::Fine, "Claude", 15, "assistant"),
    ] {
        let e = answered(&said, who, k).unwrap_err();
        assert!(e.contains(says), "{who}: expected «{says}» in: {e}");
    }
    let e = answered_to("l3_power", "1.2", "l3_power", &Said::Fine, "Pia Power", 15).unwrap_err();
    assert!(e.contains("its own release"), "{e}");
    let e = answered_to("l3_solar", "two", "l3_power", &Said::Fine, "Pia Power", 15).unwrap_err();
    assert!(e.contains("not a release's version"), "{e}");
    // Ola's key under Pia's name: it does not count.
    let e = answered(&Said::Fine, "Pia Power", 13).unwrap_err();
    assert!(
        e.contains("is not one the programme's file registers"),
        "{e}"
    );
    // A release relabelled: its fingerprint is not what it holds.
    let mut relabelled = release_of("l3_solar", "1.2");
    relabelled.meta.insert(s("fingerprint"), s("0"));
    let e = answer::answer(
        &relabelled,
        "l3_power",
        &Said::Fine,
        "Pia Power",
        &key(15),
        AT,
        APP,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("an answer is to the release its seal names"),
        "{}",
        e.message()
    );
}

/// `f` with its meta and texts as `edit` leaves them, signed again by `who`.
fn resigned(f: &File, who: &str, k: u8, edit: &dyn Fn(&mut File)) -> File {
    let mut f = f.clone();
    edit(&mut f);
    f.signatures.clear();
    let row = chain::sign(&f, FILE, who, &key(k), AT, "ok", "").unwrap();
    f.signatures.push(row);
    f
}

#[test]
fn an_answer_is_held_to_every_rule_wherever_it_is_read() {
    let pr = programme();
    let sy = systems(&pr);
    let fine = answered(&Said::Fine, "Pia Power", 15).unwrap();
    let refused = |f: &File, says: &str| {
        let e = chain::check_answer(f, &sy, &pr).unwrap_err();
        assert!(
            e.message().contains(says),
            "expected «{says}» in: {}",
            e.message()
        );
    };
    // Its own release, made by hand past `answer`.
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| {
            f.meta.insert(s("group_id"), s("l3_power"));
        }),
        "does not answer its own release",
    );
    // The systems' branch decides an objection; it does not answer.
    refused(
        &resigned(&fine, "Sam Engel", 12, &|f| {
            f.meta.insert(s("answering_group"), s("systems"));
        }),
        "the systems' branch decides an objection",
    );
    // A version that is not one.
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| {
            f.meta.insert(s("version"), s("x"));
        }),
        "version",
    );
    // Written by Pia, signed only by Dee: one writer per file.
    refused(
        &resigned(&fine, "Dee Puty", 14, &|_| {}),
        "written by Pia Power",
    );
    // Saying two things, or nothing, or with a comment riding along.
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| {
            f.texts.push(Text {
                scope: s(FILE),
                kind: s("answer"),
                body: s("object"),
            });
        }),
        "says its answer twice",
    );
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| f.texts.clear()),
        "the answer says nothing",
    );
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| {
            f.comments.push(Comment {
                scope: s(FILE),
                section: s(""),
                author: s("Ola Sun"),
                at: s(AT),
                body: s("ride along"),
                resolved: 0,
            });
        }),
        "the answer holds comment rows",
    );
    // An answer unsigned does not count.
    let mut bare = fine.clone();
    bare.signatures.clear();
    assert!(chain::check_answer(&bare, &sy, &pr).is_err());
    // A wrong kind.
    let mut other = fine.clone();
    other.meta.insert(s("file_kind"), s("issue"));
    refused(&other, "not an answer");
    // One person owning both groups does not answer one for the other.
    let mut both = group_file("systems");
    mount(&mut both, "sys_space_environment", "l3_solar", "Pia Power");
    mount(&mut both, "sys_power", "l3_power", "Pia Power");
    let both = chain::open_systems(&signed(both, "Sam Engel", &key(12)), &pr).unwrap();
    let e = chain::check_answer(&fine, &both, &pr).unwrap_err();
    assert!(
        e.message()
            .contains("Pia Power owns both l3_solar and l3_power"),
        "{}",
        e.message()
    );
    // A group nobody mounts has nobody to answer for it.
    refused(
        &resigned(&fine, "Pia Power", 15, &|f| {
            f.meta.insert(s("answering_group"), s("l3_comms"));
        }),
        "mounts no group",
    );
}

#[test]
fn an_answer_is_written_into_integration_and_never_over_another() {
    let copy = std::env::temp_dir().join(format!("vleo-answer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&copy);
    std::fs::create_dir_all(&copy).unwrap();
    let pr = programme();
    let sy = systems(&pr);
    // No release of that version in the copy: nothing to answer.
    let e = answer::answer_in_place(
        &copy,
        "l3_solar",
        "1.2",
        "l3_power",
        &Said::Fine,
        "Pia Power",
        &key(15),
        AT,
        APP,
        &sy,
        &pr,
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("your copy holds no release l3_solar 1.2"),
        "{}",
        e.message()
    );
    let releases = copy.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&releases).unwrap();
    vleo_files::sqlite::write(
        &release_of("l3_solar", "1.2"),
        &releases.join("l3_solar-1.2.vleo"),
    )
    .unwrap();
    let write = |said: &Said| {
        answer::answer_in_place(
            &copy,
            "l3_solar",
            "1.2",
            "l3_power",
            said,
            "Pia Power",
            &key(15),
            AT,
            APP,
            &sy,
            &pr,
        )
    };
    let rel = write(&Said::Fine).unwrap();
    assert_eq!(rel, "integration/l3_solar-1.2/l3_solar-1.2.l3_power.answer");
    let f = vleo_files::sqlite::read(&copy.join(&rel)).unwrap();
    chain::check_answer(&f, &sy, &pr).unwrap();
    let before = std::fs::read(copy.join(&rel)).unwrap();
    let e = write(&Said::Object(s("on second thoughts"))).unwrap_err();
    assert!(
        e.message().contains("never written over"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(copy.join(&rel)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&copy);
}
