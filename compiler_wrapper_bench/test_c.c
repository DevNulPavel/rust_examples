#include <stdio.h>
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
}

// #include <stdio.h>
// #include <stdlib.h>
// #include <unistd.h>
// int main(){
//     int pid = fork();
//     if(pid != 0){
//         execlp("clang", "clang", NULL);
//         exit(1);
//     }
//     wait(NULL);
//     return 0;
// }