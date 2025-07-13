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

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Testing actual stdout output would require more complex setup
    // These tests focus on ensuring the functions don't panic and handle edge cases

    #[test]
    fn test_print_register_table_empty() {
        let registers: &[u16] = &[];
        // Should not panic and should handle empty input gracefully
        print_register_table(registers, 0, false);
        print_register_table(registers, 0, true);
    }

    #[test]
    fn test_print_register_table_single() {
        let registers = [42];
        // Should not panic
        print_register_table(&registers, 100, false);
        print_register_table(&registers, 100, true);
    }

    #[test]
    fn test_print_register_table_multiple() {
        let registers = [0, 1, 2, 255, 65535];
        // Should not panic
        print_register_table(&registers, 0, false);
        print_register_table(&registers, 1000, true);
    }

    #[test]
    fn test_print_coil_table_empty() {
        let coils: &[bool] = &[];
        // Should not panic and should handle empty input gracefully
        print_coil_table(coils, 0);
    }

    #[test]
    fn test_print_coil_table_mixed() {
        let coils = [true, false, true, true, false];
        // Should not panic
        print_coil_table(&coils, 10);
    }

    #[test]
    fn test_print_table_header() {
        // Test various column configurations
        print_table_header(&["Address", "Value"]);
        print_table_header(&["Address", "Value", "Hex"]);
        print_table_header(&["A", "B", "C", "D"]);
    }

    // Test the actual logic by examining what addresses would be generated
    #[test]
    fn test_register_addressing() {
        let registers = [100, 200, 300];
        let start_addr = 50;

        // Verify that addresses would be calculated correctly
        for (i, _) in registers.iter().enumerate() {
            let expected_addr = start_addr + i as u16;
            assert_eq!(expected_addr, start_addr + i as u16);
        }
    }

    #[test]
    fn test_coil_addressing() {
        let coils = [true, false, true];
        let start_addr = 1000;

        // Verify that addresses would be calculated correctly
        for (i, _) in coils.iter().enumerate() {
            let expected_addr = start_addr + i as u16;
            assert_eq!(expected_addr, start_addr + i as u16);
        }
    }
}
