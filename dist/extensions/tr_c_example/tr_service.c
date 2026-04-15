#include <stdio.h>
#include <string.h>
//TODO
int main(void) {
    char buffer[8192];
    while (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        char output_str[9000] = "This is an example. Source text: ";
        strcat_s(output_str, sizeof(output_str), buffer);
        fputs(output_str, stdout);
        fflush(stdout);
    }
    return 0;
}