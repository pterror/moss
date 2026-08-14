import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import type { FC } from 'react';

interface CounterProps {
    initialCount: number;
    step?: number;
    label: string;
}

interface ButtonProps {
    onClick: () => void;
    children: React.ReactNode;
}

type Theme = 'light' | 'dark';

const Button: FC<ButtonProps> = ({ onClick, children }) => (
    <button onClick={onClick}>{children}</button>
);

const Counter: FC<CounterProps> = ({ initialCount, step = 1, label }) => {
    const [count, setCount] = useState(initialCount);
    const [theme, setTheme] = useState<Theme>('light');

    useEffect(() => {
        document.title = `${label}: ${count}`;
    }, [count, label]);

    const increment = () => setCount(c => c + step);
    const decrement = () => setCount(c => c - step);
    const reset = () => setCount(initialCount);

    return (
        <div className={`counter ${theme}`}>
            <h2>{label}</h2>
            <p>Count: {count}</p>
            <Button onClick={increment}>+</Button>
            <Button onClick={decrement}>-</Button>
            <Button onClick={reset}>Reset</Button>
        </div>
    );
};

// Classify a number as negative, zero, or positive
function classify(n: number): string {
    if (n < 0) {
        return 'negative';
    } else if (n === 0) {
        return 'zero';
    } else {
        return 'positive';
    }
}

// Namespace grouping related helpers — a real-world TS idiom for
// organizing utility functions and types without a class.
namespace Format {
    export function currency(n: number): string {
        return `$${n.toFixed(2)}`;
    }

    export class Money {
        constructor(public cents: number) {}
    }
}

enum Status {
    Idle,
    Loading,
    Done,
}

interface Comparable<T> {
    compareTo(other: T): number;
}

abstract class BaseWidget implements Comparable<BaseWidget> {
    abstract compareTo(other: BaseWidget): number;
}

class StatusWidget extends BaseWidget {
    status: Status = Status.Idle;

    compareTo(other: BaseWidget): number {
        return 0;
    }
}

async function loadPlugin() {
    // Dynamic import — common code-splitting idiom in TSX apps.
    const mod = await import('./plugins/analytics');
    return mod.default;
}

function* statusStream(): Generator<Status> {
    yield Status.Idle;
    yield Status.Loading;
}

function useLazyValue<T>(factory: () => T): T {
    // Non-null assertion before a call — common in loosely-typed interop code.
    const cache: Record<string, (() => T) | undefined> = {};
    cache['factory'] = factory;
    return cache['factory']!();
}

const eventHandlers: Record<string, () => void> = {
    onClick: () => {},
};
// Bracket/computed call target — common in event-dispatch tables.
eventHandlers['onClick']();

new Format.Money(100); // namespaced constructor — exercises reference.class via member_expression

export default Counter;
export { classify, Button };
export { Status as WidgetStatus };
export * from './shared-types';
