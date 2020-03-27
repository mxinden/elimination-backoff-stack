use crate::event::{Event, print_padded};

pub(crate) fn print_report(events: Vec<Event>) {
    let operations = split_by_operation(events);
    println!("# operations: {:?}\n", operations.len());

    let (push_ops, pop_ops) = seperate_push_and_pop(operations);
    println!("# push ops: {:?}", push_ops.len());
    println!("# pop ops: {:?}\n", pop_ops.len());

    println!("longest push op: {:?}", longest_operation(&push_ops));
    push_ops[index_longest_operation(&push_ops)].iter().for_each(print_padded);
    println!("");

    println!("longest pop op: {:?}\n", longest_operation(&pop_ops));
    pop_ops[index_longest_operation(&pop_ops)].iter().for_each(print_padded);
    println!("");
}

enum Operation {
    Push(Vec<Event>),
    Pop(Vec<Event>),
}

impl Operation {
    fn push(&mut self, e: Event) {
        match self {
            Operation::Push(events) => events.push(e),
            Operation::Pop(events) => events.push(e),
        }
    }
}

fn split_by_operation(events: Vec<Event>) -> Vec<Operation> {
    events.into_iter().fold(vec![], |mut acc, event| {
        let len = acc.len();
        match event {
            e @ Event::StartPush => acc.push(Operation::Push(vec![e])),
            e @ Event::StartPop => acc.push(Operation::Pop(vec![e])),
            e @ _ => acc[len - 1].push(e),
        };
        acc
    })
}

fn seperate_push_and_pop(operations: Vec<Operation>) -> (Vec<Vec<Event>>, Vec<Vec<Event>>) {
    operations.into_iter().fold((vec![], vec![]), |mut acc, operation| {
        match operation {
            Operation::Push(events) => acc.0.push(events),
            Operation::Pop(events) => acc.1.push(events),
        };

        acc
    })
}

fn longest_operation(operations: &Vec<Vec<Event>>) -> usize {
    operations.iter().fold(0, |acc, o| {
        if o.len() > acc {
            o.len()
        } else {
            acc
        }
    })
}

fn index_longest_operation(operations: &Vec<Vec<Event>>) -> usize {
    let (index, _) = operations.iter().enumerate().fold((0, 0), |(acc_index, acc_len), (o_index, o)| {
        if o.len() > acc_len {
            (o_index, o.len())
        } else {
            (acc_index, acc_len)
        }
    });

    index
}
