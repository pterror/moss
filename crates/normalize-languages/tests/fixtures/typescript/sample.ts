import { EventEmitter } from 'events';
import * as path from 'path';

export interface Logger {
    log(message: string): void;
    error(message: string): void;
}

// Logs to a file
@Injectable()
export class FileLogger implements Logger {
    private prefix: string;

    constructor(prefix: string) {
        this.prefix = prefix;
    }

    log(message: string): void {
        console.log(`[${this.prefix}] ${message}`);
    }

    error(message: string): void {
        console.error(`[${this.prefix}] ERROR: ${message}`);
    }
}

export function formatPath(filePath: string): string {
    const normalized = path.normalize(filePath);
    if (normalized.startsWith('/')) {
        return normalized;
    }
    return `./${normalized}`;
}

export function groupBy<T>(items: T[], key: (item: T) => string): Map<string, T[]> {
    const result = new Map<string, T[]>();
    for (const item of items) {
        const k = key(item);
        const group = result.get(k) ?? [];
        group.push(item);
        result.set(k, group);
    }
    return result;
}

// Base class + generic interface, combined extends + implements — checks that
// both @reference.class (extends) and @reference.implementation (implements,
// generic form) are found on the same class_declaration.
export interface Comparable<T> {
    compareTo(other: T): number;
}

export class Entity {
    id: number = 0;
}

export class Widget extends Entity implements Comparable<Widget> {
    // Private field + private method (# syntax) — must be found as class
    // members but never confused with public property_identifier members.
    #cache: Map<string, number> = new Map();

    #computeScore(): number {
        return this.id * 2;
    }

    compareTo(other: Widget): number {
        return this.#computeScore() - other.id;
    }

    score(): number {
        // obj.#privateMethod() call — must be captured with @call.qualifier,
        // same as a public method call.
        return this.#computeScore();
    }
}

// Namespace with a nested dotted namespace — `namespace X {}` parses as
// `internal_module`, distinct from (and far more common than) the legacy
// `module X {}` keyword form.
export namespace Shapes {
    export function describe(name: string): string {
        return `shape:${name}`;
    }

    export namespace Nested {
        export const unit = 1;
    }
}

// Closures / higher-order functions: real TS leans on these heavily, and
// closures must never be reported as function/method definitions.
export function makeCounter(start: number): () => number {
    let value = start;
    return () => {
        value += 1;
        return value;
    };
}

// async/await + promise chain idiom.
export async function fetchAndLog(loader: FileLogger): Promise<void> {
    const value = await Promise.resolve(42);
    loader.log(`fetched ${value}`);
    Promise.resolve(value)
        .then((v) => v * 2)
        .catch((err) => loader.error(String(err)));
}

// Destructuring in parameters + object/array destructuring assignment.
export function widgetSummary({ id }: Widget, [first, ...rest]: number[]): string {
    return `${id}:${first}:${rest.length}`;
}

// Optional chaining + nullish coalescing on a qualified (namespaced)
// constructor call — new ns.Ctor() must still be found as @reference.class.
export function safeDescribe(shape: { name?: string } | null): string {
    return Shapes.describe(shape?.name ?? 'unknown');
}
