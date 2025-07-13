// Helper function for table headers
pub fn print_table_header(columns: &[&str]) {
    // Print column names
    for (i, col) in columns.iter().enumerate() {
        if i == 0 {
            print!("{col:<8}");
        } else {
            print!(" {col:<6}");
        }
    }
    println!();

    // Print separator line
    for (i, _) in columns.iter().enumerate() {
        if i == 0 {
            print!("{:─<8}", "");
        } else {
            print!(" {:─<6}", "");
        }
    }
    println!();
}

pub fn print_register_table(registers: &[u16], start_addr: u16, verbose: bool) {
    if registers.is_empty() {
        return;
    }

    // Use helper for header
    if verbose {
        print_table_header(&["Address", "Value", "Hex"]);
    } else {
        print_table_header(&["Address", "Value"]);
    }

    // Print data rows
    for (i, &value) in registers.iter().enumerate() {
        let addr = start_addr + i as u16;
        if verbose {
            println!("{addr:<8} {value:<6} 0x{value:04X}");
        } else {
            println!("{addr:<8} {value:<6}");
        }
    }
}

pub fn print_coil_table(coils: &[bool], start_addr: u16) {
    if coils.is_empty() {
        return;
    }

    // Use helper for header
    print_table_header(&["Address", "Value"]);

    // Print data rows
    for (i, &value) in coils.iter().enumerate() {
        let addr = start_addr + i as u16;
        println!("{:<8} {:<6}", addr, if value { "ON" } else { "OFF" });
    }
}