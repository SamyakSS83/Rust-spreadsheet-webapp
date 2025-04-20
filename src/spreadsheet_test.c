#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <math.h>
#include "spreadsheet.h"
#include "cell.h"
#include "orderedset.h"


// Helper function to check cell values
void assert_cell_value(Spreadsheet *sheet, const char *cell_name, int expected_value, int expected_error) {
    int row, col;
    spreadsheet_parse_cell_name(sheet, cell_name, &row, &col);
    Cell *cell = sheet->cells[sheet->cols * (row - 1) + (col - 1)];
    assert(cell != NULL);
    assert(cell->value == expected_value);
    assert(cell->error == expected_error);
    printf("✓ Cell %s has value %d and error status %d as expected\n", 
           cell_name, expected_value, expected_error);
}

// Helper to safely set a cell's formula
void set_cell(Spreadsheet *sheet, const char *cell_name, const char *formula) {
    char status[256];
    spreadsheet_set_cell_value(sheet, (char *)cell_name, formula, status, sizeof(status));
    // printf("Setting %s=%s: %s\n", cell_name, formula, status);
}

// Test spreadsheet creation and basic properties
void test_spreadsheet_create() {
    printf("\n====== Testing spreadsheet_create ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    assert(sheet != NULL);
    assert(sheet->rows == 100);
    assert(sheet->cols == 100);
    assert(sheet->cells != NULL);
    printf("✓ Spreadsheet created successfully with 100x100 dimensions\n");
    
    // Test cell initialization
    Cell *cell_a1 = sheet->cells[0]; // A1
    assert(cell_a1 != NULL);
    assert(cell_a1->row == 1);
    assert(cell_a1->col == 1);
    assert(cell_a1->value == 0);
    assert(cell_a1->formula == NULL);
    assert(cell_a1->error == 0);
    printf("✓ Cell A1 initialized correctly\n");
    
    // Test a cell in the middle
    Cell *cell_j10 = sheet->cells[9 * sheet->cols + 9]; // J10
    assert(cell_j10 != NULL);
    assert(cell_j10->row == 10);
    assert(cell_j10->col == 10);
    printf("✓ Cell J10 initialized correctly\n");
    
    // Test bottom-right cell
    Cell *cell_cv100 = sheet->cells[99 * sheet->cols + 99]; // CV100
    assert(cell_cv100 != NULL);
    assert(cell_cv100->row == 100);
    assert(cell_cv100->col == 100);
    printf("✓ Cell CV100 (bottom-right) initialized correctly\n");
    
    // Cleanup
    // TODO: Implement spreadsheet_destroy
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Test column/letter conversion functions
void test_column_letter_conversion() {
    printf("\n====== Testing column/letter conversion ======\n");
    
    // Column to letter tests
    char buffer[64];
    assert(strcmp(spreadsheet_col_to_letter(1, buffer, sizeof(buffer)), "A") == 0);
    printf("✓ Column 1 converts to 'A'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(26, buffer, sizeof(buffer)), "Z") == 0);
    printf("✓ Column 26 converts to 'Z'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(27, buffer, sizeof(buffer)), "AA") == 0);
    printf("✓ Column 27 converts to 'AA'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(52, buffer, sizeof(buffer)), "AZ") == 0);
    printf("✓ Column 52 converts to 'AZ'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(100, buffer, sizeof(buffer)), "CV") == 0);
    printf("✓ Column 100 converts to 'CV'\n");

    assert(strcmp(spreadsheet_col_to_letter(703, buffer, sizeof(buffer)), "AAA") == 0);
    printf("✓ Column 703 converts to 'AAA'\n");

    assert(strcmp(spreadsheet_col_to_letter(1404, buffer, sizeof(buffer)), "BAZ") == 0);
    printf("✓ Column 1404 converts to 'BAZ'\n");

    assert(strcmp(spreadsheet_col_to_letter(703, buffer, sizeof(buffer)), "AAA") == 0);
    printf("✓ Column 703 converts to 'AAA'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(704, buffer, sizeof(buffer)), "AAB") == 0);
    printf("✓ Column 704 converts to 'AAB'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(728, buffer, sizeof(buffer)), "AAZ") == 0);
    printf("✓ Column 728 converts to 'AAZ'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(729, buffer, sizeof(buffer)), "ABA") == 0);
    printf("✓ Column 729 converts to 'ABA'\n");
    
    assert(strcmp(spreadsheet_col_to_letter(18278, buffer, sizeof(buffer)), "ZZZ") == 0);
    printf("✓ Column 18278 converts to 'ZZZ'\n");
    
    
    // Letter to column tests
    assert(spreadsheet_letter_to_col("A") == 1);
    printf("✓ 'A' converts to column 1\n");
    
    assert(spreadsheet_letter_to_col("Z") == 26);
    printf("✓ 'Z' converts to column 26\n");
    
    assert(spreadsheet_letter_to_col("AA") == 27);
    printf("✓ 'AA' converts to column 27\n");
    
    assert(spreadsheet_letter_to_col("AZ") == 52);
    printf("✓ 'AZ' converts to column 52\n");
    
    assert(spreadsheet_letter_to_col("CV") == 100);
    printf("✓ 'CV' converts to column 100\n");
    
    assert(spreadsheet_letter_to_col("AAA") == 703);
    printf("✓ 'AAA' converts to column 703\n");

    assert(spreadsheet_letter_to_col("BAZ") == 1404);
    printf("✓ 'BAZ' converts to column 1404\n");
    
    assert(spreadsheet_letter_to_col("AAB") == 704);
    printf("✓ 'AAB' converts to column 704\n");
    
    assert(spreadsheet_letter_to_col("AAZ") == 728);
    printf("✓ 'AAZ' converts to column 728\n");
    
    assert(spreadsheet_letter_to_col("ABA") == 729);
    printf("✓ 'ABA' converts to column 729\n");

    assert(spreadsheet_letter_to_col("ZZZ") == 18278);
    printf("✓ 'ZZZ' converts to column 18278\n");
    
    // Test round-trip conversion
    for (int col = 1; col <= 100; col++) {
        spreadsheet_col_to_letter(col, buffer, sizeof(buffer));
        int back = spreadsheet_letter_to_col(buffer);
        assert(col == back);
    }
    printf("✓ Round-trip conversion successful for columns 1-100\n");
}

// Test cell name helper functions
void test_cell_name_helpers() {
    printf("\n====== Testing cell name helpers ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    
    // Test cell name generation
    char buffer[64];
    
    assert(strcmp(spreadsheet_get_cell_name(1, 1, buffer, sizeof(buffer)), "A1") == 0);
    printf("✓ Cell (1,1) generates name 'A1'\n");
    
    assert(strcmp(spreadsheet_get_cell_name(10, 26, buffer, sizeof(buffer)), "Z10") == 0);
    printf("✓ Cell (10,26) generates name 'Z10'\n");
    
    assert(strcmp(spreadsheet_get_cell_name(100, 100, buffer, sizeof(buffer)), "CV100") == 0);
    printf("✓ Cell (100,100) generates name 'CV100'\n");
    
    // Test cell name parsing
    int row, col;
    
    assert(spreadsheet_parse_cell_name(sheet, "A1", &row, &col) == 1);
    assert(row == 1 && col == 1);
    printf("✓ 'A1' parses to row=1, col=1\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "Z10", &row, &col) == 1);
    assert(row == 10 && col == 26);
    printf("✓ 'Z10' parses to row=10, col=26\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "CV100", &row, &col) == 1);
    assert(row == 100 && col == 100);
    printf("✓ 'CV100' parses to row=100, col=100\n");
    
    // // Test invalid cell names
    // assert(spreadsheet_parse_cell_name(sheet, "A0", &row, &col) == 0);
    // printf("✓ 'A0' correctly identified as invalid\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "A101", &row, &col) == 0);
    printf("✓ 'A101' correctly identified as invalid (row out of bounds)\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "CW1", &row, &col) == 0);
    printf("✓ 'CW1' correctly identified as invalid (column out of bounds)\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "1A", &row, &col) == 0);
    printf("✓ '1A' correctly identified as invalid format\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "A1B", &row, &col) == 0);
    printf("✓ 'A1B' correctly identified as invalid format\n");
    
    assert(spreadsheet_parse_cell_name(sheet, "", &row, &col) == 0);
    printf("✓ Empty string correctly identified as invalid\n");
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

int std(int* arr, int n) {
    if (n <= 1) return 0;  // Avoid division by zero

    int sum = 0, mean;
    double variance = 0.0;

    // Calculate mean
    for (int i = 0; i < n; i++) {
        sum += arr[i];
    }
    mean = sum / n;

    // Calculate variance
    for (int i = 0; i < n; i++) {
        variance += (arr[i] - mean) * (arr[i] - mean);
    }
    variance /= n;

    // Return integer standard deviation (rounded)
    return (int)round(sqrt(variance));
}


void test_spreadsheet_evaluate_expression() {
    printf("\n====== Testing evaluate expression ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    // Assigning values to some cells
    sheet->cells[0]->value = 2;     // A1 = 2
    sheet->cells[1]->value = -3;    // B1 = -3
    sheet->cells[2]->value = 123;   // C1 = 123
    sheet->cells[3]->value = -234;  // D1 = -234

    Cell result;

    // Previously tested cases
    assert(spreadsheet_evaluate_expression(sheet, "-1", &result) == -1);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "+1", &result) == 1);
    assert(result.error == 0);
    assert(spreadsheet_evaluate_expression(sheet, "+2", &result) == 2);
    assert(result.error == 0);
    assert(spreadsheet_evaluate_expression(sheet, "-4", &result) == -4);
    assert(result.error == 0);

    // check max int range as well
    assert(spreadsheet_evaluate_expression(sheet, "2147483647", &result) == 2147483647);
    assert(result.error == 0);
    assert(spreadsheet_evaluate_expression(sheet, "-2147483648", &result) == -2147483648);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "00", &result) == 0);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "0098", &result) == 98);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "0090", &result) == 90);
    assert(result.error == 0);
    assert(spreadsheet_evaluate_expression(sheet, "-0090", &result) == -90);
    assert(result.error == 0);

    // Test simple arithmetic
    assert(spreadsheet_evaluate_expression(sheet, "-1*-1", &result) == 1);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1*+1", &result) == -1);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1+-1", &result) == -2);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1-+1", &result) == -2);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1/+1", &result) == -1);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1/-1", &result) == 1);
    assert(result.error == 0);

    // Test division by zero errors
    assert(spreadsheet_evaluate_expression(sheet, "1/0", &result) == 0);
    assert(result.error == 1);

    assert(spreadsheet_evaluate_expression(sheet, "2/0", &result) == 0);
    assert(result.error == 1);


    // New test cases with cell references
    assert(spreadsheet_evaluate_expression(sheet, "A1*B1", &result) == (2 * -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "A1/B1", &result) == (2 / -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "A1+B1", &result) == (2 + -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-1*B1", &result) == (-1 * -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "3*B1", &result) == (3 * -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "+3*B1", &result) == (3 * -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "B1+2", &result) == (-3 + 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "B1/0", &result) == 0);
    assert(result.error == 1);  // Division by zero

    assert(spreadsheet_evaluate_expression(sheet, "B1/-0", &result) == 0);
    assert(result.error == 1);  // Division by zero

    assert(spreadsheet_evaluate_expression(sheet, "C1-D1", &result) == (123 - (-234)));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1-34", &result) == (123 - 34));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "3/A1", &result) == (3 / 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-3/A1", &result) == (-3 / 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1/-3", &result) == (123 / -3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1/+3", &result) == (123 / 3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "D1*-4", &result) == (-234 * -4));
    assert(result.error == 0);

    // Setting B1 = 0 for division test
    sheet->cells[1]->value = 0;
    assert(spreadsheet_evaluate_expression(sheet, "A1/B1", &result) == 0);
    assert(result.error == 1);  // Division by zero

    // Reset B1 to original value
    sheet->cells[1]->value = -3;

    assert(spreadsheet_evaluate_expression(sheet, "-5+D1", &result) == (-5 + (-234)));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "D1+-6", &result) == (-234 + (-6)));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1*+2", &result) == (123 * 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "A1++3", &result) == 5);
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "+6+A1", &result) == (6 + 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "6+A1", &result) == (6 + 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "+4-A1", &result) == (4 - 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "4-A1", &result) == (4 - 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "-3-A1", &result) == (-3 - 2));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1-+3", &result) == (123 - 3));
    assert(result.error == 0);

    assert(spreadsheet_evaluate_expression(sheet, "C1--3", &result) == (123 + 3));
    assert(result.error == 0);

    
    sheet->cells[0]->value = 2;       // A1 (row 1, col 1) → index 0
    sheet->cells[100]->value = -3;    // A2 (row 2, col 1) → index 100
    sheet->cells[200]->value = 123;   // A3 (row 3, col 1) → index 200
    sheet->cells[300]->value = -234;  // A4 (row 4, col 1) → index 300
    sheet->cells[400]->value = 50;    // A5 (row 5, col 1) → index 400
    sheet->cells[500]->value = 8;     // A6 (row 6, col 1) → index 500
    sheet->cells[600]->value = -99;   // A7 (row 7, col 1) → index 600
    sheet->cells[700]->value = 7;     // A8 (row 8, col 1) → index 700
    sheet->cells[800]->value = 0;     // A9 (row 9, col 1) → index 800
    sheet->cells[900]->value = 16;    // A10 (row 10, col 1) → index 900


    assert(spreadsheet_evaluate_expression(sheet, "MIN(A1:A10)", &result) == -234);
    assert(result.error == 0);

    // MAX(A1:A10) - Expect 123
    assert(spreadsheet_evaluate_expression(sheet, "MAX(A1:A10)", &result) == 123);
    assert(result.error == 0);

    // SUM(A1:A10) - Expect sum: (2 + (-3) + 123 + (-234) + 50 + 8 + (-99) + 7 + 0 + 16) = -130
    assert(spreadsheet_evaluate_expression(sheet, "SUM(A1:A10)", &result) == -130);
    assert(result.error == 0);

    // AVG(A1:A10) - Expect average: (-130 / 10) = -13.0
    assert(spreadsheet_evaluate_expression(sheet, "AVG(A1:A10)", &result) == -13);
    assert(result.error == 0);

    // STDEV(A1:A10) - Standard deviation calculation
    int values[] = {2, -3, 123, -234, 50, 8, -99, 7, 0, 16};
    int expected_stdev = std(values,10);

    assert(spreadsheet_evaluate_expression(sheet, "STDEV(A1:A10)", &result) == expected_stdev);
    assert(result.error == 0);

    // Testing SLEEP(B1) for different values of B1
    sheet->cells[1]->value = -3;  // B1 = -3
    assert(spreadsheet_evaluate_expression(sheet, "SLEEP(B1)", &result) == -3);
    assert(result.error == 0);

    sheet->cells[1]->value = 0;  // B1 = 0
    assert(spreadsheet_evaluate_expression(sheet, "SLEEP(B1)", &result) == 0);
    assert(result.error == 0);

    sheet->cells[1]->value = 5;  // B1 = 5
    assert(spreadsheet_evaluate_expression(sheet, "SLEEP(B1)", &result) == 5);
    assert(result.error == 0);
    printf("✓ All test cases passed for evaluate expression\n");
    
    // Free allocated memory
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);

}

// Test basic cell value setting and arithmetic
void test_basic_arithmetic() {
    printf("\n====== Testing basic arithmetic ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    char status[256];
    
    // Test setting literal values
    set_cell(sheet, "A1", "5");
    assert_cell_value(sheet, "A1", 5, 0);
    
    set_cell(sheet, "A2", "10");
    assert_cell_value(sheet, "A2", 10, 0);
    
    // Test basic arithmetic
    set_cell(sheet, "B1", "A1+A2");
    assert_cell_value(sheet, "B1", 15, 0);
    
    set_cell(sheet, "B2", "A2-A1");
    assert_cell_value(sheet, "B2", 5, 0);
    
    set_cell(sheet, "B3", "A1*A2");
    assert_cell_value(sheet, "B3", 50, 0);
    
    set_cell(sheet, "B4", "A2/A1");
    assert_cell_value(sheet, "B4", 2, 0);
    
    // Test more complex expressions
    set_cell(sheet, "C1", "B1+B2");
    assert_cell_value(sheet, "C1", 20, 0);
    
    // Test error conditions
    set_cell(sheet, "C2", "A1/0");
    assert_cell_value(sheet, "C2", 0, 1); // Division by zero should set error flag
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}


char vector_contains(Vector *vec, const char *str) {
    for (int16_t i = 0; i < vec->size; i++) {
        if (strcmp(vec->data[i], str) == 0) {
            return 1; // String found
        }
    }
    return 0; // String not found
}

char cell_contains(Cell*cell,char *key){
    if (cell->dependents_initialised==0){
        return 0;
    }
    else if (cell->container==0){
       Vector* vec = cell->dependents.dependents_vector;
       return vector_contains(vec,key);
    }
    else {
        return orderedset_contains(cell->dependents.dependents_set,key);
    }
}

// Test cell references and dependencies
void test_cell_dependencies() {
    printf("\n====== Testing cell dependencies ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    
    // Set up base values
    set_cell(sheet, "A1", "5");
    set_cell(sheet, "A2", "10");
    set_cell(sheet, "A3", "20");
    set_cell(sheet, "A4", "-5");
    
    // B1 depends on A1 and A2
    set_cell(sheet, "B1", "A1 + A2");
    // assert_cell_value(sheet, "B1", 15, 0);
    
    set_cell(sheet, "A1", "20");
    // assert_cell_value(sheet, "B1", 30, 0);
    
    // C1 depends on B1
    set_cell(sheet, "C1", "B1 * 2");
    // assert_cell_value(sheet, "C1", 60, 0);
    
    set_cell(sheet, "A2", "5");
    // assert_cell_value(sheet, "B1", 25, 0);
    // assert_cell_value(sheet, "C1", 50, 0);

    // Test aggregation functions
    set_cell(sheet, "D1", "MAX(A1:A4)");
    // assert_cell_value(sheet, "D1", 20, 0);

    set_cell(sheet, "D2", "MIN(A1:A4)");
    // assert_cell_value(sheet, "D2", -5, 0);

    set_cell(sheet, "D3", "SUM(A1:A4)");
    // assert_cell_value(sheet, "D3", 40, 0);

    set_cell(sheet, "D4", "AVG(A1:A4)");
    // assert_cell_value(sheet, "D4", 10, 0); // (20+5+20-5)/4 = 10

    set_cell(sheet, "D5", "STDEV(A1:A4)");
    // Manually compute standard deviation and assert

    // Test SLEEP function
    set_cell(sheet, "E1", "SLEEP(A1)");
    // assert_cell_value(sheet, "E1", 20, 0); // Should return A1's value

    set_cell(sheet, "E2", "SLEEP(A4)");
    // assert_cell_value(sheet, "E2", -5, 0); // Should return A4's value

    // Verify dependency tracking
    Cell *cell_a1 = sheet->cells[0];  // A1
    assert(cell_contains(cell_a1, "B1"));
    assert(cell_contains(cell_a1, "D1"));
    assert(cell_contains(cell_a1, "D2"));
    assert(cell_contains(cell_a1, "D3"));
    assert(cell_contains(cell_a1, "D4"));
    assert(cell_contains(cell_a1, "D5"));
    assert(cell_contains(cell_a1, "E1"));

    Cell *cell_b1 = sheet->cells[1];  // B1
    assert(cell_contains(cell_b1, "C1"));

    // now let's change formulas and check if old depedencies are removed or not 



    // Cleanup
    for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }
    free(sheet->cells);
    free(sheet);
}


void test_cell_dependency_updates() {
    printf("\n====== Testing cell dependency updates ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);

    // Set up initial values
    set_cell(sheet, "A1", "5");
    set_cell(sheet, "A2", "10");
    set_cell(sheet, "A3", "20");
    set_cell(sheet, "A4", "-5");

    // B1 depends on A1 and A2
    set_cell(sheet, "B1", "A1 + A2");
    // assert_cell_value(sheet, "B1", 15, 0);

    // Verify dependencies
    assert(cell_contains(sheet->cells[0], "B1")); // A1 -> B1
    assert(cell_contains(sheet->cells[100], "B1")); // A2 -> B1

    // Change formula of B1 (remove dependency on A1)
    set_cell(sheet, "B1", "A2 * 2");
    // assert_cell_value(sheet, "B1", 20, 0); // (10 * 2)

    // A1 should no longer be a dependency of B1
    assert(!cell_contains(sheet->cells[0], "B1"));
    assert(cell_contains(sheet->cells[100], "B1")); // A2 should still be there

    // Change formula of B1 again (make it dependent on A3 instead)
    set_cell(sheet, "B1", "A3 + 5");
    // assert_cell_value(sheet, "B1", 25, 0);

    // A2 should no longer be a dependency of B1
    assert(!cell_contains(sheet->cells[100], "B1"));
    assert(cell_contains(sheet->cells[200], "B1")); // A3 is new dependency

    // Test dependencies on aggregation functions
    set_cell(sheet, "D1", "MAX(A1:A4)");
    set_cell(sheet, "D2", "SUM(A1:A4)");

    // Ensure all A1:A4 cells track dependencies
    for (int i = 0; i < 4; i++) {
        assert(cell_contains(sheet->cells[i*100], "D1"));
        assert(cell_contains(sheet->cells[i*100], "D2"));
    }

    // Change D1 formula (should remove dependencies from A1:A4)
    set_cell(sheet, "D1", "A1 + A2");
    // assert_cell_value(sheet, "D1", 15, 0);

    // A3 and A4 should no longer be dependencies for D1
    assert(!cell_contains(sheet->cells[200], "D1"));
    assert(!cell_contains(sheet->cells[300], "D1"));
    assert(cell_contains(sheet->cells[0], "D1"));
    assert(cell_contains(sheet->cells[100],"D1"));

    // Test SLEEP function (should not affect dependencies)
    set_cell(sheet, "E1", "SLEEP(A1)");
    assert(cell_contains(sheet->cells[0], "E1"));
    
    set_cell(sheet, "E1", "SLEEP(A2)");
    assert(!cell_contains(sheet->cells[0], "E1"));
    assert(cell_contains(sheet->cells[100], "E1"));

    // Cleanup
    for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }
    free(sheet->cells);
    free(sheet);
}
void test_topo_sort() {
    printf("\n====== Testing Topological Sort ======\n");

    // Create a spreadsheet
    Spreadsheet *sheet = spreadsheet_create(100, 100);

    // Initialize data cells A1:A10
    for (int i = 1; i <= 10; i++) {
        char cell[10];
        sprintf(cell, "A%d", i);
        set_cell(sheet, cell, "1"); // Assign values
    }

    // Set formulas in B1:B5
    set_cell(sheet, "B1", "SUM(A1:A10)");
    set_cell(sheet, "B2", "MAX(A1:A10)");
    set_cell(sheet, "B3", "MIN(A1:A10)");
    set_cell(sheet, "B4", "AVG(A1:A10)");
    set_cell(sheet, "B5", "STDEV(A1:A10)");

    // Set formulas in C1:C5
    set_cell(sheet, "C1", "SUM(B1:B5)");
    set_cell(sheet, "C2", "MAX(B1:B5)");
    set_cell(sheet, "C3", "MIN(B1:B5)");
    set_cell(sheet, "C4", "AVG(B1:B5)");
    set_cell(sheet, "C5", "STDEV(B1:B5)");

    // Run topological sort starting from A1
    Node_l *sorted_list = topo_sort(sheet, sheet->cells[0]); // Assuming A1 is at index 0

    // Expected order:
    // A1→ (B1, B2, B3, B4, B5) → (C1, C2, C3, C4, C5)

    int stage = 0; // 0 = processing A1, 1 = processing B1:B5, 2 = processing C1:C5
    Node_l *current = sorted_list;
    int counter = 0;
    while (current != NULL) {
        int row = current->data->row;
        int col = current->data->col;

        // Stage 0: Processing A1:A10
        if(counter ==0){
            assert(row==1 && col==1);
        }
        else if (counter>=1 && counter<=5){
            assert(row>=1 && row<=5 && col==2);
        }
        else if (counter>=6 && counter<=10){
            assert(row>=1 && row<=5 && col==3);

        }
        else {
            printf("Unexpected cell in topological order: (%d, %d)\n", row, col);
            assert(0);
        }
        counter++;

        current = current->next;
    }

    // Cleanup
    freeList(&sorted_list);
    for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }
    free(sheet->cells);
    free(sheet);

    printf("Topological sorting test passed!\n");
}
// Test cycle detection
void test_cycle_detection() {
    printf("\n====== Testing cycle detection ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    char status[256];
    
    // Setup initial values
    set_cell(sheet, "A1", "5");
    set_cell(sheet, "B1", "A1+10");
    
    // Try to create a direct cycle
    spreadsheet_set_cell_value(sheet, (char *)"A1", "B1+5", status, sizeof(status));
    printf("Attempting direct cycle A1=B1+5: %s\n", status);
    assert(strcmp(status, "Cycle Detected") == 0);
    
    // Ensure original value is preserved
    assert_cell_value(sheet, "A1", 5, 0);
    
    // Try to create a longer cycle
    set_cell(sheet, "C1", "B1*2");
    set_cell(sheet, "D1", "C1+1");
    
    spreadsheet_set_cell_value(sheet, (char *)"A1", "D1+2", status, sizeof(status));
    printf("Attempting longer cycle A1=D1+2: %s\n", status);
    assert(strcmp(status, "Cycle Detected") == 0);
    
    // Test indirect cycles
    set_cell(sheet, "E1", "5");
    set_cell(sheet, "F1", "E1*3");
    set_cell(sheet, "G1", "F1+H1");
    
    spreadsheet_set_cell_value(sheet, (char *)"H1", "G1/2", status, sizeof(status));
    printf("Attempting indirect cycle H1=G1/2: %s\n", status);
    assert(strcmp(status, "Cycle Detected") == 0);
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Test range functions (MIN, MAX, AVG, SUM, STDEV)
void test_range_functions() {
    printf("\n====== Testing range functions ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    
    // Setup test data
    set_cell(sheet, "A1", "10");
    set_cell(sheet, "A2", "20");
    set_cell(sheet, "A3", "30");
    set_cell(sheet, "A4", "40");
    set_cell(sheet, "A5", "50");
    
    // Test SUM
    set_cell(sheet, "B1", "SUM(A1:A5)");
    assert_cell_value(sheet, "B1", 150, 0);
    printf("✓ SUM(A1:A5) = 150\n");
    
    // Test AVG
    set_cell(sheet, "B2", "AVG(A1:A5)");
    assert_cell_value(sheet, "B2", 30, 0);
    printf("✓ AVG(A1:A5) = 30\n");
    
    // Test MIN
    set_cell(sheet, "B3", "MIN(A1:A5)");
    assert_cell_value(sheet, "B3", 10, 0);
    printf("✓ MIN(A1:A5) = 10\n");
    
    // Test MAX
    set_cell(sheet, "B4", "MAX(A1:A5)");
    assert_cell_value(sheet, "B4", 50, 0);
    printf("✓ MAX(A1:A5) = 50\n");
    
    // Test STDEV
    set_cell(sheet, "B5", "STDEV(A1:A5)");
    // STDEV is sqrt(((10-30)² + (20-30)² + (30-30)² + (40-30)² + (50-30)²)/5) = sqrt(200) ≈ 14
    assert_cell_value(sheet, "B5", 14, 0);
    printf("✓ STDEV(A1:A5) ≈ 14\n");
    
    // Test range dependency
    set_cell(sheet, "A1", "100");
    assert_cell_value(sheet, "B1", 240, 0); // SUM now 240
    assert_cell_value(sheet, "B2", 48, 0);  // AVG now 48
    
    // Test rectangular ranges
    set_cell(sheet, "B6", "15");
    set_cell(sheet, "B7", "25");
    set_cell(sheet, "C1", "SUM(A1:B7)"); // This includes A1-A5, B6-B7
    assert_cell_value(sheet, "C1", 716, 0); 
    
    // Test edge cases
    set_cell(sheet, "C2", "MIN(A1:A1)");
    assert_cell_value(sheet, "C2", 100, 0);
    
    set_cell(sheet, "C3", "SUM(Z1:Z5)"); // Empty range
    assert_cell_value(sheet, "C3", 0, 0);
    
    // Invalid range format
    set_cell(sheet, "C4", "SUM(A5:A1)"); // Inverted range
    char status[256];
    spreadsheet_set_cell_value(sheet, (char *)"C5", "SUM(A5:A1)", status, sizeof(status));
    printf("Attempting invalid range SUM(A5:A1): %s\n", status);
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Test the SLEEP function
void test_sleep_function() {
    printf("\n====== Testing SLEEP function ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    
    // Test with literal
    printf("Testing SLEEP(1) - should pause for 1 second...\n");
    set_cell(sheet, "A1", "SLEEP(1)");
    assert_cell_value(sheet, "A1", 1, 0);
    
    // Test with cell reference
    set_cell(sheet, "B1", "2");
    printf("Testing SLEEP(B1) - should pause for 2 seconds...\n");
    set_cell(sheet, "A2", "SLEEP(B1)");
    assert_cell_value(sheet, "A2", 2, 0);
    
    // Test with zero
    set_cell(sheet, "A3", "SLEEP(0)");
    assert_cell_value(sheet, "A3", 0, 0);
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Test edge cases and error handling
void test_edge_cases() {
    printf("\n====== Testing edge cases ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    char status[256];
    
    // Test reference to non-existent cell
    set_cell(sheet, "A1", "Z99");
    assert_cell_value(sheet, "A1", 0, 0); // Non-existent cells have value 0
    
    // Test division by zero in various ways
    set_cell(sheet, "B1", "0");
    set_cell(sheet, "A2", "10/0");
    assert_cell_value(sheet, "A2", 0, 1); // Error flag should be set
    
    set_cell(sheet, "A3", "10/B1");
    assert_cell_value(sheet, "A3", 0, 1);
    
    // Test propagation of errors
    set_cell(sheet, "A4", "A3+5");
    assert_cell_value(sheet, "A4", 0, 1); // Error propagates
    
    // Test references to cells with errors
    set_cell(sheet, "A5", "MIN(A2:A4)");
    assert_cell_value(sheet, "A5", 0, 1);
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Test commands validation
void test_command_validation() {
    printf("\n====== Testing command validation ======\n");
    Spreadsheet *sheet = spreadsheet_create(100, 100);
    
    // Test valid commands
    char *cell = (char *)"A1";
    char *formula = (char *)"10";
    assert(is_valid_command(sheet, &cell, &formula) == 1);
    printf("✓ 'A1=10' is valid\n");
    
    formula = (char *)"B1+C1";
    assert(is_valid_command(sheet, &cell, &formula) == 1);
    printf("✓ 'A1=B1+C1' is valid\n");
    
    formula = (char *)"SUM(A1:Z10)";
    assert(is_valid_command(sheet, &cell, &formula) == 1);
    printf("✓ 'A1=SUM(A1:Z10)' is valid\n");
    
    formula = (char *)"SLEEP(5)";
    assert(is_valid_command(sheet, &cell, &formula) == 1);
    printf("✓ 'A1=SLEEP(5)' is valid\n");
    
    // Test invalid commands
    // cell = (char *)"A0"; // Invalid row
    // formula = (char *)"10";
    // assert(is_valid_command(sheet, &cell, &formula) == 0);
    // printf("✓ 'A0=10' is invalid\n");
    
    //TODO: Fix this
    cell = (char *)"A1";
    formula = (char *)"B1++C1"; // Invalid arithmetic
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=B1++C1' is invalid\n");
    
    formula = (char *)"NOFUNCTION(A1)"; // Invalid function
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=NOFUNCTION(A1)' is invalid\n");
    
    formula = (char *)"SUM(Z10:A1)"; // Inverted range
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=SUM(Z10:A1)' is invalid\n");

    // Test invalid commands
    cell = (char *)"B1";
    formula = (char *)"-A11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'B1=-A11' is invalid\n");

    cell = (char *)"C1";
    formula = (char *)"A1*-B1";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'C1=A1*-B1' is invalid\n");

    cell = (char *)"D1";
    formula = (char *)"-B11*-C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D1=-B11*-C11' is invalid\n");

    cell = (char *)"D2";
    formula = (char *)"-B11*+C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D2=-B11*+C11' is invalid\n");

    cell = (char *)"D3";
    formula = (char *)"-B11+-C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D3=-B11+-C11' is invalid\n");

    cell = (char *)"D4";
    formula = (char *)"-B11-+C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D4=-B11-+C11' is invalid\n");

    cell = (char *)"D5";
    formula = (char *)"-B11/+C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D5=-B11/+C11' is invalid\n");

    cell = (char *)"D6";
    formula = (char *)"-B11/-C11";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'D6=-B11/-C11' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"B120";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=B120' is invalid\n");

    cell = (char *)"ZZ10";
    formula = (char *)"B12";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'ZZ10=B12' is invalid\n");

    cell = (char *)"E02";
    formula = (char *)"9";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'E02=9' is invalid\n");

    cell = (char *)"E0";
    formula = (char *)"98";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'E0=98' is invalid\n");

    cell = (char *)"A2";
    formula = (char *)"MAX((A1:C3))";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A2=MAX((A1:C3))' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"SLEEP()";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=SLEEP()' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"MAX()";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=MAX()' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"==5";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1==5' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"SLEEP(C1+D1)";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=SLEEP(C1+D1)' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"SLEEP(C1+2)";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=SLEEP(C1+2)' is invalid\n");

    cell = (char *)"E1";
    formula = (char *)"--D1";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'E1=--D1' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"MAZ(B1:C5)";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=MAZ(B1:C5)' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"--5";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=--5' is invalid\n");

    cell = (char *)"A1";
    formula = (char *)"-4+++5";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ 'A1=-4+++5' is invalid\n");

    cell = (char *)"";
    formula = (char *)"";
    assert(is_valid_command(sheet, &cell, &formula) == 0);
    printf("✓ Empty cell and formula are invalid\n");
    
    // Cleanup
     for (int r = 1; r <= 100; r++)
    {
        for (int c = 1; c <= 100; c++)
        {
            cell_destroy(sheet->cells[100*(r-1)+(c-1)]);

        }
    }

    free(sheet->cells);
    free(sheet);
}

// Run all tests
int main() {
    printf("Starting spreadsheet unit tests\n");
    
    test_spreadsheet_create();
    test_column_letter_conversion();
    test_cell_name_helpers();
    test_spreadsheet_evaluate_expression();
    test_basic_arithmetic();
    test_cell_dependencies();
    test_cell_dependency_updates();
    test_topo_sort();
    test_cycle_detection();
    test_range_functions();
    test_sleep_function();
    test_edge_cases();
    test_command_validation();
    
    printf("\nAll tests completed successfully!\n");
    return 0;
}