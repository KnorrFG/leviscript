use std::io::{stdout, Stdout, Write};
use std::iter;

use anyhow::{anyhow, bail, Result};
use crossterm::{self as ct, terminal};
use leviscript_lib::core::*;
use leviscript_lib::parser::{self, PestErrVariant, PestError, Span};
use leviscript_lib::vm;
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(PartialEq, Clone)]
enum UserCommand {
    Next,
    LastCommand,
    ShowStack,
    ShowData,
    ShowStackAt(usize),
    ShowDataAt(usize),
    Quit,
}

pub fn run(
    bc: ByteCode,
    debug_info: DebugInformation,
    opcodes: &[OpCode],
    spans: &[Span],
    src: &str,
) -> Result<()> {
    let mut runner = Runner::new(bc);
    runner.reset_pc();
    let mut rl = DefaultEditor::new()?;
    let mut last_cmd = None;

    use UserCommand::*;
    loop {
        // print_next_instruction(final_bc, im_bc, pc);
        render_state(&runner, &debug_info, opcodes, src)?;
        stdout().flush()?;
        let mut cmd = read_line(&mut rl)?;
        if cmd == UserCommand::LastCommand && last_cmd.is_some() {
            cmd = last_cmd.clone().unwrap();
        }
        match &cmd {
            LastCommand => {
                // This is only reached, if there was no last command, in which case it's
                // a noop
            }
            Next => match unsafe { runner.step() } {
                StepResult::Ok => {}
                StepResult::Done(_) => return Ok(()),
                StepResult::Err(vm::Error::Runtime(msg)) => {
                    let ast_id = runner.pc_to_ast_id(&debug_info);
                    bail!(
                        "Runtime error: {}",
                        PestError::new_from_span(
                            PestErrVariant::<parser::Rule>::CustomError { message: msg },
                            spans[ast_id]
                        )
                    );
                }
                StepResult::Err(e) => bail!("VM-Error: {}", e),
            },
            ShowStack => {
                for (i, elem) in runner.mem.stack.iter().enumerate().rev() {
                    println!("{}: {:?}", i, elem);
                }
            }
            ShowData => {
                for (i, elem) in runner.mem.data_seg.iter().enumerate() {
                    println!("{}: {:?}", i, elem);
                }
            }
            ShowStackAt(i) => {
                if *i < runner.mem.stack.len() {
                    println!("{:?}", runner.mem.stack[*i]);
                } else {
                    println!("Invalid stack index");
                }
            }
            ShowDataAt(i) => {
                if *i < runner.mem.data_seg.len() {
                    println!("{:?}", runner.mem.data_seg[*i]);
                } else {
                    println!("Invalid data index");
                }
            }
            Quit => return Ok(()),
        }
        last_cmd = Some(cmd);
    }
}

fn read_line(rl: &mut DefaultEditor) -> Result<UserCommand> {
    loop {
        let line = rl.readline("> ");
        use ReadlineError::*;
        match line {
            Ok(line) => match parse_line(&line) {
                Ok(cmd) => return Ok(cmd),
                Err(e) => eprintln!("Error: {}", e),
            },
            Err(Interrupted | Eof) => return Ok(UserCommand::Quit),
            Err(other) => return Err(other.into()),
        }
    }
}

fn parse_line(line: &str) -> Result<UserCommand> {
    use UserCommand::*;
    let line = line.trim();
    let elems: Vec<_> = line.split_whitespace().collect();

    if elems.len() > 0 {
        match elems[0] {
            "n" | "next" => Ok(Next),
            "s" | "show" => parse_show(&elems[1..]),
            "q" | "quit" => Ok(Quit),
            _ => Err(anyhow!("Invalid Command")),
        }
    } else {
        Ok(LastCommand)
    }
}

fn parse_show(elems: &[&str]) -> Result<UserCommand> {
    if elems.len() == 0 {
        Err(anyhow!("show needs an argument"))
    } else if elems.len() == 1 {
        match elems[0] {
            "s" | "stack" => Ok(UserCommand::ShowStack),
            "d" | "data" => Ok(UserCommand::ShowData),
            _ => Err(anyhow!("Invalid word after show")),
        }
    } else if elems.len() == 3 && elems[1] == "at" {
        let mker: Box<dyn Fn(usize) -> UserCommand> = match elems[0] {
            "s" | "stack" => Box::new(UserCommand::ShowStackAt),
            "d" | "data" => Box::new(UserCommand::ShowDataAt),
            _ => bail!("Invalid cmd"),
        };
        Ok(mker(elems[2].parse()?))
    } else {
        bail!("Invalid Command")
    }
}

fn print_next_instruction(runner: &Runner, debug_info: &DebugInformation, opcodes: &[OpCode]) {
    let instruction_index = runner.current_bc_index(debug_info);
    let instructions = opcodes.iter().enumerate();
    println!("Next instructions:");
    for (i, inst) in instructions.skip(instruction_index).take(5) {
        println!("{}: {:?}", i, inst);
    }
}

struct Rect {
    w: u16,
    h: u16,
    x: u16,
    y: u16,
}

struct Rects {
    input: Rect,
    bc: Rect,
    src: Rect,
    stack: Rect,
    heap: Rect,
    data: Rect,
}

impl Rect {
    pub fn render(
        &self,
        stdout: &mut Stdout,
        lines: impl IntoIterator<Item = String>,
    ) -> Result<()> {
        let mut counter = 0;
        let wu = self.w as usize;
        for (i, line) in lines.into_iter().take(self.h.into()).enumerate() {
            ct::queue!(stdout, ct::cursor::MoveTo(self.x, self.y + i as u16))?;
            if wu > line.len() {
                print!("{}{}", line, vec![" "; wu - line.len()].join(""));
            } else {
                print!("{}", &line[..wu]);
            }
            counter += 1;
        }

        while counter < self.h {
            ct::queue!(stdout, ct::cursor::MoveTo(self.x, self.y + counter))?;
            print!("{}", vec![" "; wu].join(""));
            counter += 1;
        }
        Ok(())
    }
}

fn render_state(
    runner: &Runner,
    debug_info: &DebugInformation,
    opcodes: &[OpCode],
    src: &str,
) -> Result<()> {
    let curr_cursor = ct::cursor::position()?;
    let term_size = terminal::size()?;
    let rects = compute_rects(term_size);
    let stdout = &mut stdout();
    render_bc(stdout, &rects.bc, runner, debug_info, opcodes)?;
    render_stack(stdout, &rects.stack, &runner.mem.stack)?;
    render_heap(stdout, &rects.heap, &runner.mem.heap)?;
    render_data(stdout, &rects.data, &runner.mem.data_seg)?;
    render_src(stdout, &rects.src, &src)?;
    ct::queue!(stdout, ct::cursor::MoveTo(curr_cursor.0, curr_cursor.1))?;
    Ok(())
}

fn render_src(stdout: &mut Stdout, rect: &Rect, src: &str) -> Result<()> {
    rect.render(stdout, src.lines().map(|x| x.into()))?;
    Ok(())
}

fn render_data(stdout: &mut Stdout, rect: &Rect, data: &Vec<ComptimeValue>) -> Result<()> {
    let lines = std::iter::once("Data:".into()).chain(
        data.iter()
            .enumerate()
            .map(|(i, d)| format!("{}: {:?}", i, d)),
    );
    rect.render(stdout, lines)?;
    Ok(())
}

fn render_stack(stdout: &mut Stdout, rect: &Rect, stack: &vm::Stack) -> Result<()> {
    let lines_offset = rect.h as usize - stack.len();
    let stack_lines = stack
        .iter()
        .enumerate()
        .rev()
        .map(|(i, entry)| format!("{}: {:?}", i, entry));
    if lines_offset > 0 {
        rect.render(
            stdout,
            std::iter::repeat("".into())
                .take(lines_offset)
                .chain(stack_lines),
        )?;
    } else {
        rect.render(stdout, stack_lines)?;
    }
    Ok(())
}

fn render_heap(stdout: &mut Stdout, rect: &Rect, heap: &Heap<RuntimeValue>) -> Result<()> {
    let lines = iter::once("Heap:".to_string()).chain(
        heap.iter()
            .map(|entry| format!("{:#X}: {:?}", entry as *const RuntimeValue as usize, entry)),
    );
    rect.render(stdout, lines)?;
    Ok(())
}

fn render_bc(
    stdout: &mut Stdout,
    bc_rect: &Rect,
    runner: &Runner,
    debug_info: &DebugInformation,
    opcodes: &[OpCode],
) -> Result<()> {
    let lines = opcodes
        .iter()
        .enumerate()
        .skip(runner.current_bc_index(debug_info))
        .map(|(i, inst)| format!("{}: {:?}", i, inst));
    bc_rect.render(stdout, lines)
}

fn compute_rects((term_w, term_h): (u16, u16)) -> Rects {
    let width14 = term_w / 4;
    let width12 = term_w / 2;
    let width34 = term_w * 3 / 4;
    let height45 = term_h * 4 / 5;
    let height15 = term_h / 5;
    let height12 = term_h / 2;
    let height13 = term_h / 3;
    let height23 = height13 * 2;

    Rects {
        input: Rect {
            x: 0,
            y: height45,
            w: width34,
            h: height15,
        },
        src: Rect {
            x: 0,
            y: 0,
            w: width12,
            h: height45,
        },
        bc: Rect {
            x: width12,
            y: 0,
            w: width14,
            h: height45,
        },
        stack: Rect {
            x: width34,
            y: 0,
            w: width14,
            h: height13,
        },
        heap: Rect {
            x: width34,
            y: height13,
            w: width14,
            h: height13,
        },
        data: Rect {
            x: width34,
            y: height23,
            w: width14,
            h: height13,
        },
    }
}
