int side_effect() {
    return 100;
}

int main() {
    if (1 || side_effect())
        return 1;
    return 0;
}
