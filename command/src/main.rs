use std::collections::VecDeque;

//Reciever
#[derive(Debug, Default)]
struct Document {
    text: String,
}

impl Document {
    fn new() -> Self {
        Self::default()
    }

    fn append(&mut self, text: &str) {
        self.text.push_str(text)
    }

    fn replace_first(&mut self, old: &str, new: &str) -> Result<(), String> {
        if !self.text.contains(old) {
            return Err(format!("pattern not found: {old}"));
        }
        self.text = self.text.replacen(old, new, 1);
        Ok(())
    }
}

// Command
type Command = Box<dyn FnOnce(&mut Document) -> Result<(), String>>;

// Constructors
fn append(text: String) -> Command {
    Box::new(move |document: &mut Document| {
        document.append(&text);
        Ok(())
    })
}

fn replace_first(old: String, new: String) -> Command {
    Box::new(move |document: &mut Document| document.replace_first(&old, &new))
}

// Invoker
#[derive(Default)]
struct CommandQueue {
    pending: VecDeque<Command>,
}

impl CommandQueue {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, command: Command) {
        self.pending.push_back(command);
    }

    fn len(&self) -> usize {
        self.pending.len()
    }

    fn run(&mut self, document: &mut Document) -> Result<(), String> {
        while let Some(command) = self.pending.pop_front() {
            command(document)?;
        }
        Ok(())
    }
}

fn main() {
    let mut document = Document::new();
    let mut queue = CommandQueue::new();
    queue.push(append("foo".into()));
    queue.push(append("barrr".into()));
    queue.push(replace_first("rrr".into(), "zzz".into()));
    println!("commands to run: {}", queue.len());
    queue.run(&mut document).unwrap();
    println!("{:?}", document);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_enqueue_does_not_mutate_immediately() {
        let document = Document::new();
        let mut queue = CommandQueue::new();
        queue.push(append("foo".to_string()));
        assert!(document.text.is_empty());
    }

    #[test]
    fn test_enqueue_order() {
        let mut document = Document::new();
        let mut queue = CommandQueue::new();
        queue.push(append("foo-".to_string()));
        queue.push(append("bar-".to_string()));
        queue.push(append("baz".to_string()));
        queue.run(&mut document).unwrap();
        assert_eq!(document.text, "foo-bar-baz");
    }

    #[test]
    fn test_replace_first() {
        let mut document = Document::new();
        let mut queue = CommandQueue::new();
        queue.push(append("foobar".to_string()));
        queue.push(replace_first("bar".to_string(), "baz".to_string()));
        queue.run(&mut document).unwrap();
        assert_eq!(document.text, "foobaz");
    }

    #[test]
    fn test_command_fail_leaves_queue() {
        let mut document = Document::new();
        let mut queue = CommandQueue::new();
        queue.push(replace_first("bar".to_string(), "baz".to_string()));
        queue.push(append("I'm still here".to_string()));
        assert_eq!(queue.len(), 2);
        assert!(queue.run(&mut document).is_err());
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_scope_of_owned_command_param() {
        let mut document = Document::new();
        let mut queue = CommandQueue::new();
        {
            let param = "Lil' scope".to_string();
            queue.push(append(param));
        }
        queue.run(&mut document).unwrap();
        assert_eq!(document.text, "Lil' scope");
    }
}
