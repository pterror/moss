<?php

namespace App\Collections;

use App\Models\User;
use Illuminate\Support\Collection;
use Countable;
use Traversable;
require_once __DIR__ . '/bootstrap.php';

interface Comparable {
    public function compareTo(mixed $other): int;
}

trait Loggable {
    protected array $log = [];

    public function record(string $message): void {
        $this->log[] = $message;
    }
}

class Stack implements Comparable, Countable {
    use Loggable;

    private array $items = [];

    public function push(mixed $item): void {
        array_push($this->items, $item);
        $this->record("pushed");
    }

    public function pop(): mixed {
        if (empty($this->items)) {
            throw new \UnderflowException("Stack is empty");
        }
        return array_pop($this->items);
    }

    public function peek(): mixed {
        if (empty($this->items)) {
            return null;
        }
        return end($this->items);
    }

    public function isEmpty(): bool {
        return empty($this->items);
    }

    public function size(): int {
        return count($this->items);
    }

    public function count(): int {
        return $this->size();
    }

    public function compareTo(mixed $other): int {
        return $this->size() <=> $other->size();
    }
}

class BoundedStack extends Stack {
    public function __construct(private readonly int $limit) {
        parent::__construct();
    }

    public function push(mixed $item): void {
        if ($this->size() >= $this->limit) {
            throw new \OverflowException("Stack is full");
        }
        parent::push($item);
    }
}

enum Direction: string {
    case Up = 'up';
    case Down = 'down';

    public function opposite(): self {
        return match ($this) {
            self::Up => self::Down,
            self::Down => self::Up,
        };
    }
}

/**
 * Classify a number as negative, zero, or positive.
 */
#[Pure]
function classify(int $n): string {
    if ($n < 0) {
        return "negative";
    } elseif ($n === 0) {
        return "zero";
    } else {
        return "positive";
    }
}

function sumEvens(array $numbers): int {
    $total = 0;
    foreach ($numbers as $n) {
        if ($n % 2 === 0 && $n > 0) {
            $total += $n;
        }
    }
    return $total;
}

// Namespaced function call.
function describeDirection(Direction $d): string {
    return match ($d) {
        Direction::Up => "going up",
        Direction::Down => "going down",
    };
}

$stack = new Stack();
$stack->push(1);
$stack->push(2);
echo $stack->pop() . "\n";
echo classify(-5) . "\n";
echo sumEvens([1, 2, 3, 4, 5]) . "\n";

// Static method call.
$bounded = BoundedStack::class;

// Closures and arrow functions.
$double = fn(int $x): int => $x * 2;
$logger = function (string $msg) use ($stack): void {
    $stack->push($msg);
};

// First-class callable syntax.
$lengthFn = strlen(...);
$pushFn = $stack->push(...);

// Namespace-qualified function call.
$formatted = \App\Collections\classify(3);
