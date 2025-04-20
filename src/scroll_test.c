#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include "spreadsheet.h"
#include "cell.h"

// A simple function to send commands to the spreadsheet, similar to main.c's command handling
void execute_command(Spreadsheet *sheet, const char *cmd, char *status) {
    if(strlen(cmd) == 1 && (cmd[0] == 'w' || cmd[0] == 'a' || cmd[0] == 's' || cmd[0] == 'd')) {
        // Move view - copy logic from main.c
        if(cmd[0] == 'w' && sheet->view_row > 0) {
            int new_view = sheet->view_row - 10;
            sheet->view_row = (new_view < 0) ? 0 : new_view;
        } else if(cmd[0] == 's' && sheet->view_row < sheet->rows - 19) {
            int max_row = (sheet->rows > 10) ? sheet->rows - 10 : 0;
            int new_view = sheet->view_row + 10;
            sheet->view_row = (new_view > max_row) ? max_row : new_view;
        } else if(cmd[0] == 'a' && sheet->view_col > 0) {
            int new_col = sheet->view_col - 10;
            sheet->view_col = (new_col < 0) ? 0 : new_col;
        } else if(cmd[0] == 'd' && sheet->view_col < sheet->cols - 19) {
            int max_col = (sheet->cols > 10) ? sheet->cols - 10 : 0;
            int new_col = sheet->view_col + 10;
            sheet->view_col = (new_col > max_col) ? max_col : new_col;
        }
        strcpy(status, "ok");
    } else if(strncmp(cmd, "scroll_to ", 10) == 0) {
        // Parse scroll_to command
        const char *cell_name = cmd + 10;
        int row, col;
        if(!spreadsheet_parse_cell_name(sheet, cell_name, &row, &col)) {
            strcpy(status, "invalid cell");
        } else {
            sheet->view_row = row - 1;
            sheet->view_col = col - 1;
            strcpy(status, "ok");
        }
    } else {
        strcpy(status, "invalid command");
    }
}

// Helper to set cell values
void set_cell(Spreadsheet *sheet, const char *cell_name, const char *formula) {
    char status[256];
    spreadsheet_set_cell_value(sheet, (char *)cell_name, formula, status, sizeof(status));
}

int main() {
    printf("Starting interactive scroll test\n");
    
    // Create a 25x25 spreadsheet (matching your test case)
    Spreadsheet *sheet = spreadsheet_create(25, 25);
    
    // Fill the spreadsheet with values for easy visual testing
    printf("Filling spreadsheet with test data...\n");
    for (int i = 1; i <= 25; i++) {
        for (int j = 1; j <= 25; j++) {
            char cell_name[10];
            spreadsheet_get_cell_name(i, j, cell_name, sizeof(cell_name));
            char formula[20];
            sprintf(formula, "%d", i * 100 + j); // Value will be row*100+col
            set_cell(sheet, cell_name, formula);
        }
    }
    
    char status[64];
    strcpy(status, "ok");
    
    printf("\n=== SCROLL TEST INTERACTIVE SIMULATION ===\n");
    printf("Initial view (top-left corner):\n");
    spreadsheet_display(sheet);
    
    // Test a series of commands directly using the main.c logic
    const char *commands[] = {
        "s",                // scroll down
        "d",                // scroll right
        "s",                // scroll down more
        "scroll_to P20",    // jump to cell P20
        "w",                // scroll up
        "a",                // scroll left
    };
    
    int num_commands = sizeof(commands) / sizeof(commands[0]);
    
    for (int i = 0; i < num_commands; i++) {
        printf("\nExecuting command: '%s'\n", commands[i]);
        execute_command(sheet, commands[i], status);
        printf("Status: %s\n", status);
        printf("View position: row=%d, col=%d\n", sheet->view_row, sheet->view_col);
        spreadsheet_display(sheet);
    }
    
    // Try boundary cases
    printf("\n=== TESTING BOUNDARY CASES ===\n");
    
    // Scroll to a corner
    printf("\nScrolling to top-left (A1):\n");
    execute_command(sheet, "scroll_to A1", status);
    spreadsheet_display(sheet);
    
    // Try to scroll beyond left boundary
    printf("\nTrying to scroll left beyond boundary ('a'):\n");
    execute_command(sheet, "a", status);
    printf("Status: %s, view_row=%d, view_col=%d\n", status, sheet->view_row, sheet->view_col);
    
    // Scroll to bottom-right corner
    printf("\nScrolling to bottom-right (Y25):\n");
    execute_command(sheet, "scroll_to Y25", status);
    spreadsheet_display(sheet);
    
    // Try to scroll beyond right/bottom boundary
    printf("\nTrying to scroll down beyond boundary ('s'):\n");
    execute_command(sheet, "s", status);
    printf("Status: %s, view_row=%d, view_col=%d\n", status, sheet->view_row, sheet->view_col);
    
    printf("\nTrying to scroll right beyond boundary ('d'):\n");
    execute_command(sheet, "d", status);
    printf("Status: %s, view_row=%d, view_col=%d\n", status, sheet->view_row, sheet->view_col);
    
    // Cleanup
    destroySpreadsheet(sheet);
    
    printf("\nScroll test completed successfully!\n");
    return 0;
}