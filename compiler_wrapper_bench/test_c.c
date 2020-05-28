/*#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h> 
int main(int argc, char* argv[]){
    putenv("CCACHE_PREFIX=/usr/local/bin/distcc");

    const int prefixes = 2;
    char** args_new = (char**)malloc(sizeof(char*) * (argc + prefixes));
    int index = 0;
    args_new[index++] = "ccache";
    args_new[index++] = "clang";
    for(int i = index; i < argc; i++){
        args_new[i] = argv[i];
    }

    if (execvp(args_new[0], args_new) != 0){
        return -1;    
    }
    return errno;
}*/

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h> 
int main(){
    int pid = fork();
    if(pid == 0){
        if (execlp("clang", "clang", NULL) == 0){
            exit(0);
            return 0;
        }else{
            exit(errno);
            return errno;
        }
    }
    int status;
    wait(&status);
    return WEXITSTATUS(status);
}