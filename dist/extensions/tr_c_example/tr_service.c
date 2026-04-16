#include <stdio.h>
#include <string.h>

int main(void) {
    char buffer[8192];
    while (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        char output_str[9000] = "This is an example. Source text: ";
        strcat_s(output_str, sizeof(output_str), buffer);
        fputs(output_str, stdout);
        fflush(stdout);
    }
    //ERROR CODES:
    //73 - Language error (unsupported) 
    //53 - service error
    //0 - success (no errors)
    return 0;
}