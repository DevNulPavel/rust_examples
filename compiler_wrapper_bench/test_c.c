#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main(){
    int pid = fork();
    if(pid != 0){
        execlp("clang", "clang", NULL);
        exit(1);
    }
    wait(NULL);
    return 0;
}