use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TaskOptions {
    #[structopt(name = "add", about = "Add a task")]
    Add,

    #[structopt(name = "rm", about = "Remove a task")]
    Remove,

    #[structopt(name = "ls", about = "List current tasks todo")]
    List,

    #[structopt(name = "finish", about = "Mark a task as completed")]
    Complete,

}