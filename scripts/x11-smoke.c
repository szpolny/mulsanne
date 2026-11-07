/* Small X11 fixture and input-region probe for smoke-linux.mjs. */
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/extensions/shape.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
int main(int argc, char **argv) {
    Display *d = XOpenDisplay(NULL);
    if (!d) return 1;
    if (argc == 3 && strcmp(argv[1], "area") == 0) {
        int count = 0, ordering = 0;
        XRectangle *rects = XShapeGetRectangles(d, strtoul(argv[2], NULL, 0), ShapeInput, &count, &ordering);
        unsigned long area = 0;
        for (int i = 0; i < count; ++i) area += (unsigned long)rects[i].width * rects[i].height;
        XFree(rects);
        printf("%lu\n", area);
        XCloseDisplay(d);
        return 0;
    }
    Window w = XCreateSimpleWindow(d, DefaultRootWindow(d), 80, 80, 1200, 900, 0, 0, 0);
    XClassHint hint = { "steam_app_2399420", "steam_app_2399420" };
    XSetClassHint(d, w, &hint);
    XStoreName(d, w, "Mulsanne smoke game");
    XMapWindow(d, w);
    XFlush(d);
    printf("%lu\n", w);
    fflush(stdout);
    for (;;) pause();
}
