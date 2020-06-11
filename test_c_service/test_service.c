#include <stdio.h>
#include <stdlib.h>
#include <systemd/sd-daemon.h>
#include <unistd.h>
#include <string.h>

// build and run with: clang -o test_service test_service.c $(pkg-config libsystemd --libs) && ./test_service

int main(int argc, char **argv) {
  printf("Amount of args: %d\n", argc);
  printf("Result of sd_booted: %d\n", sd_booted());
  printf("Result of sd_listen_fds: %d\n", sd_listen_fds(0));
  char **names;
  int fds = sd_listen_fds_with_names(0, &names);
  printf("Result of sd_listen_fds_with_names: %d\n", fds);

  printf("FD names:\n");
  for( int i = 0; i < fds; i++) {
      printf("\t%s\n", names[i]);
      if (!strcmp("SockeyMcSocketFace", names[i])) {
          printf("\tFD #%d  TCP socket test (should be 1): %d\n", i, sd_is_socket_inet(i+3,  AF_INET,  SOCK_STREAM, -1, 0));
      }
      if (!strcmp("SockeyMcSocketFace2", names[i])) {
          printf("\tFD #%d  UDP socket test (should be 1): %d\n", i, sd_is_socket_inet(i+3,  AF_INET,  SOCK_DGRAM, -1, 0));
      }
      if (!strcmp("SockeyMcSocketFace3", names[i])) {
          printf("\tFD #%d  Fifo socket test (should be 1): %d\n", i, sd_is_fifo(i+3, "./sockets/cservice.fifo"));
      }
  }

  printf("Env var VAR1: %s\tVAR2: %s\tVAR3: %s\n", getenv("VAR1"), getenv("VAR2"), getenv("VAR3"));

  int res = sd_notify(0, "READY=1\n");
  printf("Result of sd_notify: %d\n", res);
  printf("Result of sd_notify: %d\n",
         sd_notify(0, "STATUS=New status from C service\n"));
         
  fflush(stdout);
  while (1) {
    sd_notify(0, "READY=1\nSTATUS=still looping\n");
    sleep(1);
  }
}
