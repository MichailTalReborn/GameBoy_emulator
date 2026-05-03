mod lr35902;

use lr35902::{ArithmeticTarget, CPU, Instruction};

fn main() {
    let mut cpu = CPU::new();

    cpu.registers.a = 5;
    cpu.registers.b = 3;

    cpu.execute(Instruction::ADD(ArithmeticTarget::B));

    println!("A = {}", cpu.registers.a);
}
