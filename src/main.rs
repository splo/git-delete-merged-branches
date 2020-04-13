use git2::{Repository, BranchType};

fn main() {
    let repo = Repository::open_from_env().unwrap();
    let head = repo.head().unwrap();
    let head_commit = repo.reference_to_annotated_commit(&head).unwrap();
    println!("head: {:?}", &head_commit.id());
    for result in repo.branches(Some(BranchType::Local)).unwrap() {
        let (branch, _) = result.unwrap();
        let branch_name = branch.name().unwrap().unwrap();
        println!("- {}", branch_name);
        let branch_commit = repo.reference_to_annotated_commit(&branch.into_reference()).unwrap();
        println!("    commit: {:?}", &branch_commit.id());
        let array = repo.merge_bases(head_commit.id(), branch_commit.id()).unwrap();
        println!("    IDs: {:?}", array);
    };
}
