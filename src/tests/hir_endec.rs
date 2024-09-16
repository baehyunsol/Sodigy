// TODO
// 1. what I first tried was
//    a. compile a file, dump its hir as json and endec-ed binary
//    b. construct a session from the endec-ed binary
//    c. dump its hir as json again, with the constructed session
//    d. compare the 2 json files
// 2. the problem was that the endec-roundtrip slightly changes the order of items
//    I could fix that but the fix would make the overall compilation slower
// 3. so my plan is to
//    a. compile a file directly from code to binary
//    b. compile a file from code to hir, then endec-ed hir session to mir
//    c. check if the two programs emit the same output
