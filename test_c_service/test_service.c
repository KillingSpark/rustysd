#include <systemd/sd-daemon.h>
#include <stdio.h>

int main() {
    printf("Result of sd_booted: %d\n", sd_booted());
    printf("Result of sd_listen_fds: %d\n", sd_listen_fds(0));
    printf("Result of sd_notify: %d\n", sd_notify(0, "STATUS=New status from C service"));
}