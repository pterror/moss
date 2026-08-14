#import <Foundation/Foundation.h>
#import "Shape.h"
@import Contacts;

@protocol Drawable <NSObject>
- (void)draw;
@optional
- (NSString *)debugDescription;
@end

#pragma mark - Point

@interface Point : NSObject <Drawable>
@property (nonatomic) double x;
@property (nonatomic) double y;
@property (nonatomic, copy) NSString *label;
@property (nonatomic, copy) void (^onChange)(double newX, double newY);
- (instancetype)initWithX:(double)x y:(double)y;
+ (instancetype)origin;
@end

@implementation Point
@synthesize label = _label;
// Initializes a Point with x and y coordinates.
- (instancetype)initWithX:(double)x y:(double)y {
    self = [super init];
    if (self) {
        _x = x;
        _y = y;
        self.onChange = ^(double newX, double newY) {
            NSLog(@"moved to %f,%f", newX, newY);
        };
    }
    return self;
}

+ (instancetype)origin {
    return [[self alloc] initWithX:0.0 y:0.0];
}

- (void)draw {
    NSLog(@"drawing point at %f,%f", _x, _y);
}
@end

#pragma mark - Point (Formatting)

// Category: adds formatting behavior to Point without subclassing.
@interface Point (Formatting)
- (NSString *)formattedDescription;
@end

@implementation Point (Formatting)
- (NSString *)formattedDescription {
    return [NSString stringWithFormat:@"(%f, %f)", self.x, self.y];
}
@end

#pragma mark - Circle

@interface Circle : NSObject <Drawable, NSCopying>
@property (nonatomic) double radius;
- (instancetype)initWithRadius:(double)radius;
- (double)area;
@end

@implementation Circle
- (instancetype)initWithRadius:(double)radius {
    self = [super init];
    if (self) {
        _radius = radius;
    }
    return self;
}

- (double)area {
    return M_PI * _radius * _radius;
}

- (void)draw {
    NSLog(@"drawing circle r=%f", _radius);
}

- (id)copyWithZone:(NSZone *)zone {
    return [[Circle allocWithZone:zone] initWithRadius:_radius];
}
@end

double distance(Point *a, Point *b) {
    double dx = b.x - a.x;
    double dy = b.y - a.y;
    return sqrt(dx * dx + dy * dy);
}

NSString *classify(int n) {
    if (n < 0) {
        return @"negative";
    } else if (n == 0) {
        return @"zero";
    } else {
        return @"positive";
    }
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        Point *p1 = [[Point alloc] initWithX:3.0 y:4.0];
        Point *p2 = [[Point alloc] initWithX:0.0 y:0.0];
        NSLog(@"distance: %f", distance(p1, p2));
        Circle *c = [[Circle alloc] initWithRadius:5.0];
        NSLog(@"area: %f", [c area]);
        NSLog(@"%@", classify(-3));
    }
    return 0;
}
