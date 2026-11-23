#include <iostream>
#include "../beta/lib.h" // Example of including a header file from a local dependency.
#include "locallib.h"

using namespace std;

int main()
{
    cout << "Hello World" << endl;
    writeline("This is an example of using a local dependency.");
    writeline(example_function());
    return 0;
}
