use git2::{
    Branch, BranchType, Config, Cred, Direction, ProxyOptions, Remote, RemoteCallbacks, Repository,
};
use std::io;
use std::io::BufRead;

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> Result<(), git2::Error> {
    let repository = &Repository::open_from_env()?;
    let main_branch = find_main_branch(repository, None)?;
    let mut merged_branch_names = Vec::new();
    let main_branch_oid = main_branch.get().target().unwrap();
    let main_branch_name = get_branch_name(&main_branch)?;
    for result in repository.branches(Some(BranchType::Local))? {
        let (branch, _) = result?;
        let branch_name = get_branch_name(&branch)?;
        let is_main_branch = branch_name == main_branch_name;
        let branch_ref = branch.get();
        let branch_commit = repository.reference_to_annotated_commit(&branch_ref)?;
        let merge_base = repository.merge_base(main_branch_oid, branch_commit.id())?;
        let is_merged = merge_base == branch_commit.id();
        if is_merged && !is_main_branch {
            merged_branch_names.push(branch_name);
        }
    }
    let message = match merged_branch_names.len() {
        std::usize::MIN..=0 => "No branch to delete",
        1 => "Do you want to delete this branch?",
        _ => "Do you want to delete the following branches?",
    };
    println!("{}", message);
    if !merged_branch_names.is_empty() {
        merged_branch_names
            .iter()
            .for_each(|name| println!("  {}", name));
        let input = io::stdin().lock().lines().next().unwrap().unwrap();
        if input.to_lowercase().as_str() == "y" {
            merged_branch_names
                .iter()
                .for_each(|name| match delete_branch(repository, name) {
                    Ok(()) => (),
                    Err(e) => println!("Could not delete branch {}: {}", name, e),
                });
        }
    }
    Ok(())
}

fn find_main_branch<'a>(
    repository: &'a Repository,
    name_opt: Option<&str>,
) -> Result<Branch<'a>, git2::Error> {
    let main_branch_name = find_remote(repository, name_opt)?
        .map(|remote_name| repository.find_remote(&*remote_name).unwrap())
        .map(|remote| {
            find_main_branch_name_from_remote(&repository.config().unwrap(), remote).unwrap()
        })
        .unwrap_or_else(|| String::from("master"));
    let main_branch = repository.find_branch(main_branch_name.as_str(), BranchType::Local)?;
    Ok(main_branch)
}

fn find_remote(
    repository: &Repository,
    name_opt: Option<&str>,
) -> Result<Option<String>, git2::Error> {
    let remote_name: String;
    if let Some(name) = name_opt {
        remote_name = String::from(name);
    } else {
        let result = repository.remotes();
        let remotes = result?;
        let length = remotes.len();
        if length == 1 {
            remote_name = String::from(remotes.get(0).unwrap());
        } else if length == 0 {
            return Ok(None);
        } else {
            remote_name = String::from("origin");
        }
    }
    Ok(Some(remote_name))
}

fn find_main_branch_name_from_remote(
    config: &Config,
    mut remote: Remote,
) -> Result<String, git2::Error> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, username_from_url, _allowed_types| {
        Cred::credential_helper(config, url, username_from_url)
    });
    remote.connect_auth(
        Direction::Fetch,
        Some(callbacks),
        Some(ProxyOptions::default()),
    )?;
    let rh = remote
        .list()?
        .iter()
        .find(|rh| rh.symref_target().is_some())
        .ok_or_else(|| git2::Error::from_str("Could not find a remote head"))?;
    let remote_symref_target = rh.symref_target().unwrap();
    let main_branch_name: String = remote_symref_target.chars().skip(11).collect();
    remote.disconnect()?;
    Ok(main_branch_name)
}

fn get_branch_name(branch: &Branch) -> Result<String, git2::Error> {
    Ok(String::from(branch.name()?.ok_or_else(|| {
        git2::Error::from_str("Invalid branch name")
    })?))
}

fn delete_branch(repository: &Repository, name: &str) -> Result<(), git2::Error> {
    for result in repository.branches(Some(BranchType::Local))? {
        let (mut branch, _) = result?;
        let branch_name = get_branch_name(&branch)?;
        if name == branch_name.as_str() {
            branch.delete()?
        }
    }
    Ok(())
}
