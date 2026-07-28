import { EventEmitter } from 'events';
import path from 'path';
import { readFileSync, writeFileSync } from 'fs';

// A stack data structure
@sealed
class Stack extends EventEmitter {
    #items = [];

    constructor(name) {
        super();
        this.name = name;
    }

    push(item) {
        this.#items.push(item);
        this.emit('push', item);
        return this;
    }

    pop() {
        if (this.isEmpty()) {
            return undefined;
        }
        const item = this.#items.pop();
        this.emit('pop', item);
        return item;
    }

    isEmpty() {
        return this.#items.length === 0;
    }

    size() {
        return this.#items.length;
    }
}

function classify(n) {
    if (n < 0) {
        return 'negative';
    } else if (n === 0) {
        return 'zero';
    } else {
        return 'positive';
    }
}

const sumArray = (nums) => {
    let total = 0;
    for (const n of nums) {
        total += n;
    }
    return total;
};

function fibonacci(n) {
    if (n <= 1) return n;
    let a = 0, b = 1;
    for (let i = 2; i <= n; i++) {
        [a, b] = [b, a + b];
    }
    return b;
}

const stack = new Stack('demo');
stack.push(1).push(2).push(3);
console.log(classify(-1));
console.log(sumArray([1, 2, 3, 4, 5]));
console.log(fibonacci(10));
const resolved = path.resolve('./sample.js');
console.log(resolved);

// Mixin pattern: `extends Mixin(Base)` — a call_expression as the superclass
// expression, common in the class-mixin idiom.
function Serializable(Base) {
    return class extends Base {
        serialize() {
            return JSON.stringify(this);
        }
    };
}

class SerializableStack extends Serializable(Stack) {
    // Private method (# syntax) — must be found as a class member, and its
    // call site (this.#peek()) must be found with @call.qualifier, same as
    // a public method call.
    #peek() {
        return this.#items[this.#items.length - 1];
    }

    // Computed method name — the key expression, not a plain identifier.
    ['top']() {
        return this.#peek();
    }

    // Static method — still a plain property_identifier name.
    static create(name) {
        return new SerializableStack(name);
    }

    // Getter/setter — property_identifier names, same shape as a method.
    get topValue() {
        return this.#peek();
    }
}

// Generator function.
function* range(start, end) {
    for (let i = start; i < end; i++) {
        yield i;
    }
}

// async/await + promise chain.
async function loadAndSum(urls) {
    const values = await Promise.all(urls.map((u) => fetch(u)));
    return values
        .reduce((acc, v) => acc + v, 0);
}

// Destructuring in parameters (object + array + defaults).
function describeEntry({ name, count = 0 }, [first, ...rest]) {
    return `${name}:${count}:${first}:${rest.length}`;
}

// Tagged template literal — a call_expression whose `arguments` field is a
// bare template_string, not the usual `arguments` node.
function html(strings, ...values) {
    return strings.reduce((acc, s, i) => acc + s + (values[i] ?? ''), '');
}
const page = html`<h1>${resolved}</h1>`;

// CommonJS interop still supported alongside ES modules in real JS codebases.
const { statSync } = require('fs');

// Computed/bracket call — dynamic method dispatch by name.
const dispatch = { classify };
dispatch['classify'](0);

for (const n of range(0, 3)) {
    console.log(n);
}
