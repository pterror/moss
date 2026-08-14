// Thrift IDL sample file

namespace py sample
namespace java com.example.sample
namespace go sample

include "shared.thrift"
cpp_include "custom_types.h"

// A unique identifier type
typedef string UUID
typedef list<string> Tags

// User account status
enum Status {
  ACTIVE = 1,
  INACTIVE = 2,
  BANNED = 3,
}

// A user in the system, with a cpp codegen customization annotation.
struct User {
  1: required UUID id,
  2: required string name (cpp.type = "std::string"),
  3: required string email,
  4: optional Status status = Status.ACTIVE,
  5: optional list<string> roles,
  6: optional map<string, string> metadata,
  7: optional set<UUID> connections,
} (cpp.type = "UserRecord")

// A union of possible search results -- exactly one field is set.
union SearchResult {
  1: User user,
  2: string errorMessage,
}

// Exception thrown when user is not found
exception UserNotFound {
  1: string message,
}

// Exception thrown for internal errors, marked transient (retryable).
transient exception SystemError {
  1: string detail,
}

const i32 MAX_PAGE_SIZE = 100
const list<string> DEFAULT_ROLES = ["viewer"]

// Base service for administrative operations.
service AdminService {
  oneway void audit(1: string action),
}

// Service for managing users, extending the base admin service and using
// a facebook-style prefix annotation.
@fb.ThriftService
service UserService extends AdminService {
  // Retrieve a user by ID
  User getUser(1: UUID id) throws (1: UserNotFound notFound),
  list<User> listUsers(1: i32 limit = 20),
  map<UUID, User> getUsersById(1: set<UUID> ids),
  void deleteUser(1: UUID id) throws (1: UserNotFound notFound, 2: SystemError sysErr),
  SearchResult search(1: string query),
}
