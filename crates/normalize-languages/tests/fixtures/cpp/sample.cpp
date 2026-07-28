#include <iostream>
#include <memory>
#include <stdexcept>
#include <vector>

using namespace std::literals;

namespace shapes {

// Real-world idiom: destructor + move-only smart-pointer member, requiring
// an explicit (if trivial) destructor — the kind of class where forgetting
// destructor extraction entirely (the bug this fixture guards against)
// would silently blind `normalize view` to every RAII type in a codebase.
class Shape {
public:
    explicit Shape(std::string label) : label_(std::move(label)) {}
    virtual ~Shape();
    virtual double area() const = 0;
    const std::string &label() const { return label_; }

private:
    std::string label_;
};

class Circle : public Shape {
public:
    Circle(std::string label, double radius)
        : Shape(std::move(label)), radius_(radius) {}
    double area() const override { return 3.14159 * radius_ * radius_; }

private:
    double radius_;
};

} // namespace shapes

template <typename T>
class Stack {
public:
    // Pushes an item onto the stack.
    void push(const T& item) {
        items.push_back(item);
    }

    T pop() {
        if (items.empty()) {
            throw std::underflow_error("Stack is empty");
        }
        T top = items.back();
        items.pop_back();
        return top;
    }

    const T& peek() const {
        if (items.empty()) {
            throw std::underflow_error("Stack is empty");
        }
        return items.back();
    }

    bool empty() const { return items.empty(); }
    std::size_t size() const { return items.size(); }

    // Operator overload — the pre-fix query never tagged any operator
    // overload, inline or out-of-line, anywhere in a C++ codebase.
    Stack &operator+=(const T &item) {
        push(item);
        return *this;
    }

private:
    std::vector<T> items;
};

// Out-of-line destructor, the routine non-header-only pattern the pre-fix
// query dropped entirely (function_declarator.declarator = destructor_name).
// Defined inside a reopened `namespace shapes { ... }` block (idiomatic
// style) rather than fully-qualified as `shapes::Shape::~Shape`, so the
// qualifier stays single-level (`Shape::~Shape`) — the deeply-nested
// multi-level-qualifier combination is a separate, rarer case not covered
// here (see cpp.tags.scm's own comments for what depth is verified).
//
// Note: `Shape::~Shape() = default;` (rather than `{}`) at namespace scope
// is a documented arborium-cpp/tree-sitter-cpp grammar ambiguity — it
// parses as an `expression_statement` (a call to `Shape::~Shape` assigned
// to an identifier named "default"), not a `function_definition`, so no
// query can recover a tag from it. Verified via `normalize syntax ast`;
// this is a grammar limitation, not a query gap, so `{}` is used here
// instead to keep the fixture representative of what the grammar can
// actually resolve.
namespace shapes {
Shape::~Shape() {}
} // namespace shapes

[[nodiscard]] std::string classify(int n) {
    if (n < 0) {
        return "negative";
    } else if (n == 0) {
        return "zero";
    } else {
        return "positive";
    }
}

int sum_evens(const std::vector<int>& numbers) {
    int total = 0;
    for (int n : numbers) {
        if (n % 2 == 0) {
            total += n;
        }
    }
    return total;
}

template <typename T>
T identity(T x) {
    return x;
}

int main() {
    Stack<int> s;
    s.push(10);
    s.push(20);
    s += 30; // operator overload call, verified separately by
             // cpp_calls_completeness (calls itself aren't operator-tagged;
             // this just documents the idiom compiles against the fixture)
    std::cout << s.pop() << "\n";
    std::cout << classify(-3) << "\n";
    std::vector<int> nums = {1, 2, 3, 4, 5};
    std::cout << sum_evens(nums) << "\n";

    // Smart pointer + polymorphism, auto deduction, range-based for: the
    // idioms that lean hardest on destructor/virtual-dispatch extraction.
    std::vector<std::unique_ptr<shapes::Shape>> figures;
    figures.push_back(std::make_unique<shapes::Circle>("circle", 2.0));
    for (const auto &figure : figures) {
        std::cout << figure->label() << ": " << figure->area() << "\n";
    }

    // Turbofish-equivalent explicit template argument call — direct C++
    // analogue of Rust's turbofish gap fixed via cpp.calls.scm.
    int doubled = identity<int>(21) * 2;

    // Lambda with capture, immediately invoked — closures must never be
    // mistaken for function/method definitions.
    auto adder = [doubled](int x) { return x + doubled; };
    std::cout << adder(1) << "\n";

    return 0;
}
