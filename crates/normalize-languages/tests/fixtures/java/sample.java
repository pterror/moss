import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.function.Function;

public class TaskQueue implements Comparable<TaskQueue>, java.io.Serializable {
    private List<String> tasks;
    private int capacity;

    public TaskQueue(int capacity) {
        this.tasks = new ArrayList<>();
        this.capacity = capacity;
    }

    public boolean enqueue(String task) {
        if (tasks.size() >= capacity) {
            return false;
        }
        tasks.add(task);
        return true;
    }

    public String dequeue() {
        if (tasks.isEmpty()) {
            return null;
        }
        return tasks.remove(0);
    }

    // Returns the size
    @Override
    public int size() {
        return tasks.size();
    }

    public static String classify(int n) {
        if (n < 0) {
            return "negative";
        } else if (n == 0) {
            return "zero";
        } else {
            return "positive";
        }
    }

    @Override
    public int compareTo(TaskQueue other) {
        return Integer.compare(this.capacity, other.capacity);
    }

    // Real-world idiom: iterator-chain pipeline over the task list.
    public long countLongTasks() {
        return tasks.stream()
            .filter(t -> t.length() > 10)
            .map(String::length)
            .count();
    }

    // Generic method: identity-like helper.
    public static <T> T firstOrDefault(List<T> items, T fallback) {
        return items.isEmpty() ? fallback : items.get(0);
    }

    // Nested static class using a generic + qualified supertype.
    public static class PriorityTaskQueue extends TaskQueue implements java.util.Comparator<String> {
        public PriorityTaskQueue(int capacity) {
            super(capacity);
        }

        @Override
        public int compare(String a, String b) {
            return a.compareTo(b);
        }
    }

    // Anonymous class idiom.
    public Runnable asRunnable() {
        return new Runnable() {
            @Override
            public void run() {
                dequeue();
            }
        };
    }

    // Lambda + functional-interface idiom (must not be reported as a
    // definition.method/definition.function — closures are not method_declaration).
    public Function<String, Integer> lengthFn() {
        return String::length;
    }
}

interface Processor {
    String process(String input);

    // Default and static interface methods (Java 8+).
    default String processOrEmpty(String input) {
        return input == null ? "" : process(input);
    }

    static Processor identity() {
        return input -> input;
    }
}

// Nested module-shaped grouping (top-level classes in one compilation unit).
class Shapes {
    static String describe(int sides) {
        switch (sides) {
            case 3 -> {
                return "triangle";
            }
            case 4 -> {
                return "quadrilateral";
            }
            default -> {
                return "polygon";
            }
        }
    }
}

// Enum with a constructor and a method (Java enums can carry both).
enum Color {
    RED(0xFF0000),
    GREEN(0x00FF00),
    BLUE(0x0000FF);

    private final int rgb;

    Color(int rgb) {
        this.rgb = rgb;
    }

    int rgb() {
        return rgb;
    }
}

// Record (Java 16+): a real-world data-carrier idiom.
record Point(int x, int y) {
    int manhattanDistance() {
        return Math.abs(x) + Math.abs(y);
    }
}
