% Sample MATLAB file

classdef Shape < handle
    % A base class for geometric shapes
    properties
        Color = 'blue'
    end

    methods
        function obj = Shape(color)
            if nargin > 0
                obj.Color = color;
            end
        end

        function area = computeArea(obj)
            area = 0;
        end

        function display(obj)
            fprintf('Shape: color=%s, area=%g\n', obj.Color, obj.computeArea());
        end
    end

    methods (Abstract)
        % Abstract method declaration: signature only, no body.
        area = describe(obj)
    end
end

classdef Circle < Shape & matlab.mixin.Copyable
    % Multi-superclass subclass; exercises a Circle object built on top of
    % Shape's superclass-qualified constructor call.
    properties
        Radius
    end

    methods
        function obj = Circle(radius)
            obj = obj@Shape('red');
            obj.Radius = radius;
        end

        function area = computeArea(obj)
            area = pi * obj.Radius ^ 2;
        end
    end
end

function result = factorial(n)
% Compute factorial of n
    import matlab.io.*
    import matlab.net.http.RequestMessage
    if n <= 1
        result = 1;
    else
        result = n * factorial(n - 1);
    end
end

function total = sumArray(arr)
% Return sum of all elements in arr
    total = 0;
    for k = 1:length(arr)
        total = total + arr(k);
    end
end

function filtered = filterPositive(arr)
% Return only positive elements of arr
    filtered = [];
    for k = 1:length(arr)
        if arr(k) > 0
            filtered(end + 1) = arr(k); %#ok<AGROW>
        end
    end
end

function result = classifyNumber(n)
% Classify a number as negative, zero, or positive
    switch sign(n)
        case -1
            result = 'negative';
        case 0
            result = 'zero';
        otherwise
            result = 'positive';
    end
end

function results = describeShapes(shapes)
% Real-world idioms: cell arrays, structs, anonymous functions, while,
% try/catch, and a dynamic-field ("indirect access") call dispatch.
    results = {};
    handlers = struct('circle', @(s) s.computeArea(), 'shape', @(s) s.computeArea());
    i = 1;
    while i <= length(shapes)
        shape = shapes{i};
        try
            kind = 'shape';
            area = handlers.(kind)(shape);
            results{end + 1} = sprintf('%s: %g', class(shape), area);
        catch err
            warning('describeShapes:failed', 'skipping shape: %s', err.message);
        end
        i = i + 1;
    end
end
