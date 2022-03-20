use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub struct Program<'a> {
    source: &'a str,
    pub instructions: Vec<Instruction>,
    functions: HashMap<String, usize>,
}

impl<'a> Program<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            instructions: Vec::new(),
            functions: HashMap::new(),
        }
    }

    fn parse(&mut self) {
        // Clean code and make instructions
        self.instructions = self.source.lines()
            .map(|x| {
                let mut clean = String::new();
                // Remove comment
                match x.find(';') {
                    Some(com_pos) => clean.push_str(&x[..com_pos]),
                    None => clean.push_str(&x)
                }
                Instruction::from(clean.trim().to_string())
            })
            .collect();

        // Find functions
        for (index, instruction) in self.instructions.iter().enumerate() {
            if let Instruction::Function(name) = instruction {
                self.functions.insert(name.to_owned(), index + 1);
            }
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Mov(String, String),
    Inc(String),
    Dec(String),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
    Function(String),
    Call(String),
    Cmp(String, String),
    Jmp(String),
    Jne(String),
    Je(String),
    Jge(String),
    Jg(String),
    Jle(String),
    Jl(String),
    Msg(Vec<String>),
    Ret,
    End,
    Nop,
}

impl From<String> for Instruction {
    fn from(raw_instruction: String) -> Self {
        if raw_instruction == "" {
            return Instruction::Nop;
        }

        let args: Vec<&str> = raw_instruction.split_whitespace().collect();
        let raw_params = raw_instruction.replace(&args[0], "");

        let params: Vec<&str> = raw_params.trim()
            .split(',')
            .fold(vec![], |mut res, curr: &str| {
                res.push(curr.trim());
                res
            });

        match args[0] {
            "mov" => Instruction::Mov(params[0].to_string(), params[1].to_string()),
            "inc" => Instruction::Inc(params[0].to_string()),
            "dec" => Instruction::Dec(params[0].to_string()),
            "add" => Instruction::Add(params[0].to_string(), params[1].to_string()),
            "sub" => Instruction::Sub(params[0].to_string(), params[1].to_string()),
            "mul" => Instruction::Mul(params[0].to_string(), params[1].to_string()),
            "div" => Instruction::Div(params[0].to_string(), params[1].to_string()),
            "call" => Instruction::Call(params[0].to_string()),
            "cmp" => Instruction::Cmp(params[0].to_string(), params[1].to_string()),
            "jmp" => Instruction::Jmp(params[0].to_string()),
            "jne" => Instruction::Jne(params[0].to_string()),
            "je" => Instruction::Je(params[0].to_string()),
            "jge" => Instruction::Jge(params[0].to_string()),
            "jg" => Instruction::Jg(params[0].to_string()),
            "jle" => Instruction::Jle(params[0].to_string()),
            "jl" => Instruction::Jl(params[0].to_string()),
            "msg" => Instruction::Msg(params.iter().map(|x| x.to_string()).collect()),
            "ret" => Instruction::Ret,
            "end" => Instruction::End,
            other => if other.ends_with(":") {
                Instruction::Function(other.trim_matches(':').to_string())
            } else {
                Instruction::Nop
            }
        }
    }
}

pub struct Interpreter<'a> {
    stack: Vec<usize>,
    register: HashMap<String, i64>,
    rip: usize,
    zf: u8,
    cf: u8,
    out: String,
    pub program: Program<'a>,
}

impl Display for Interpreter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let delimiter = "-".repeat(20);

        // Registers
        writeln!(f, "Registers:")?;
        writeln!(f, "{}", delimiter)?;
        for (key, value) in self.register.iter() {
            writeln!(f, "{:<5}: {:<10}", key, value)?;
        }
        writeln!(f, "{}", delimiter)?;

        // Stack
        write!(f, "\nStack:")?;
        if !self.stack.is_empty() {
            writeln!(f, "{}", delimiter)?;
            for (num, line) in self.stack.iter().enumerate() {
                writeln!(f, "{:<10}: {:<10}", num, line)?;
            }
            writeln!(f, "{}", delimiter)?;
        } else {
            write!(f, "Empty\n")?;
        }

        // Flags
        write!(f, "\nFlags:")?;
        write!(f, "\n{}\n", delimiter)?;
        write!(f, "{:<2}: {:<10}\n", "ZF", self.zf)?;
        write!(f, "{:<2}: {:<10}", "CF", self.cf)?;
        write!(f, "\n{}\n", delimiter)?;

        // Output
        write!(f, "\nOutput: {}\n", self.out)?;

        // Pointer
        write!(f, "\nRIP: {}\n", self.rip)?;
        Ok(())
    }
}

impl<'a> Interpreter<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            stack: Vec::new(),
            register: HashMap::new(),
            rip: 0,
            zf: 0,
            cf: 0,
            out: String::new(),
            program: Program::new(source),
        }
    }

    pub fn interpret(input: &'a str) -> (Self, Option<String>) {
        let mut interpreter = Interpreter::new(input);
        interpreter.program.parse();
        let output = interpreter.run();
        (interpreter, output)
    }

    fn run(&mut self) -> Option<String> {
        loop {
            match self.program.instructions.get(self.rip)? {
                Instruction::Mov(dst, src) => {
                    let src_value = self.constant_or_register(src);
                    *self.register.entry(dst.into()).or_insert(0) = src_value;
                    self.rip += 1;
                }

                Instruction::Inc(dst) => {
                    *self.register.entry(dst.into()).or_insert(0) += 1;
                    self.rip += 1;
                }

                Instruction::Dec(dst) => {
                    *self.register.entry(dst.into()).or_insert(0) -= 1;
                    self.rip += 1;
                }

                Instruction::Add(dst, src) => {
                    let src_value = self.constant_or_register(src);
                    *self.register.entry(dst.into()).or_insert(0) += src_value;
                    self.rip += 1;
                }

                Instruction::Sub(dst, src) => {
                    let src_value = self.constant_or_register(src);
                    *self.register.entry(dst.into()).or_insert(0) -= src_value;
                    self.rip += 1;
                }

                Instruction::Mul(dst, src) => {
                    let src_value = self.constant_or_register(src);
                    *self.register.entry(dst.into()).or_insert(0) *= src_value;
                    self.rip += 1;
                }

                Instruction::Div(dst, src) => {
                    let src_value = self.constant_or_register(src);
                    *self.register.entry(dst.into()).or_insert(0) /= src_value;
                    self.rip += 1;
                }

                Instruction::Call(label) => {
                    self.stack.push(self.rip + 1);
                    self.rip = *self.program.functions.get(label).unwrap();
                }

                Instruction::Cmp(dst, src) => {
                    // Reset flags
                    self.zf = 0;
                    self.cf = 0;

                    let dst_value = self.constant_or_register(dst);
                    let src_value = self.constant_or_register(src);

                    if dst_value == src_value {
                        self.zf = 1;
                    } else if dst_value < src_value {
                        self.cf = 1;
                    }

                    self.rip += 1;
                }

                Instruction::Jmp(label) => {
                    self.rip = *self.program.functions.get(label).unwrap();
                }

                Instruction::Jne(label) => {
                    if self.zf != 1 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Je(label) => {
                    if self.zf == 1 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Jge(label) => {
                    if self.zf == 1 || self.cf == 0 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Jg(label) => {
                    if self.zf == 0 && self.cf == 0 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Jle(label) => {
                    if self.cf == 1 || self.zf == 1 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Jl(label) => {
                    if self.cf == 1 {
                        self.rip = *self.program.functions.get(label).unwrap();
                    } else {
                        self.rip += 1;
                    }
                }

                Instruction::Msg(args) => {
                    let mut opened = false;
                    // Concat arguments
                    let res: String = args.iter().map(|i| {
                        if i == "'" {
                            if !opened {
                                opened = !opened;
                                String::from(",")
                            } else {
                                opened = !opened;
                                String::from(" ")
                            }
                        } else if i.contains("'") {
                            i.trim_matches('\'').to_string()
                        } else {
                            self.constant_or_register(i).to_string()
                        }
                    }).collect();

                    self.out = res;
                    self.rip += 1;
                }

                Instruction::Ret => {
                    self.rip = self.stack.pop().unwrap();
                }

                Instruction::End => {
                    return Some(self.out.to_owned());
                }

                Instruction::Function(_) | Instruction::Nop => {
                    self.rip += 1;
                }
            }
        }
    }

    fn constant_or_register(&self, src: &str) -> i64 {
        match src.parse::<i64>() {
            Ok(r) => r,
            _ => *self.register.get(src).unwrap_or(&0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_interpreter() {
        let programs_list = &[
            "\n; My first program\nmov  a, 5\ninc  a\ncall function\nmsg  '(5+1)/2 = ', a    ; output message\nend\n\nfunction:\n    div  a, 2\n    ret\n",
            "\nmov   a, 5\nmov   b, a\nmov   c, a\ncall  proc_fact\ncall  print\nend\n\nproc_fact:\n    dec   b\n    mul   c, b\n    cmp   b, 1\n    jne   proc_fact\n    ret\n\nprint:\n    msg   a, '! = ', c ; output text\n    ret\n",
            "\ncall  func1\ncall  print\nend\n\nfunc1:\n    call  func2\n    ret\n\nfunc2:\n    ret\n\nprint:\n    msg 'This program should return null'\n",
            "\n            mov a, 173   ; instruction mov a, 173\n            mov k, 88   ; instruction mov k, 88\n            call func\n            msg 'Random result: ', o\n            end\n            func:\n              cmp a, k\n              jne exit\n              mov o, a\n              add o, k\n              ret\n            ; Do nothing\n            exit:\n              msg 'Do nothing'",
            "\n            mov q, 86   ; instruction mov q, 86\n            mov m, 73   ; instruction mov m, 73\n            call func\n            msg 'Random result: ', g\n            end\n            func:\n              cmp q, m\n              jl exit\n              mov g, q\n              div g, m\n              ret\n            ; Do nothing\n            exit:\n              msg 'Do nothing'"
        ];

        assert_eq!(Some(String::from("(5+1)/2 = 3")), Interpreter::interpret(&programs_list[0]).1);
        assert_eq!(Some(String::from("5! = 120")), Interpreter::interpret(&programs_list[1]).1);
        assert_eq!(None, Interpreter::interpret(&programs_list[2]).1);
        assert_eq!(None, Interpreter::interpret(&programs_list[3]).1);
        assert_eq!(Some(String::from("Random result: 1")), Interpreter::interpret(&programs_list[4]).1);
    }
}