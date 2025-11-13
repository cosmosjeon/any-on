-- Demo Data Seed Script
-- Creates orphaned data (user_id = NULL) for UI development
-- This data will be automatically claimed by the first user who logs in

-- Clean up existing demo data first (if any)
DELETE FROM tasks WHERE user_id IS NULL;
DELETE FROM projects WHERE user_id IS NULL;

-- Insert demo projects
INSERT OR REPLACE INTO projects (id, user_id, name, git_repo_path, setup_script, dev_script, cleanup_script, copy_files, created_at, updated_at)
VALUES 
  (X'11111111111111111111111111111111', NULL, 'Demo Project - E-Commerce', '/tmp/demo-ecommerce', 'npm install', 'npm run dev', 'npm run cleanup', NULL, datetime('now', '-30 days'), datetime('now', '-1 days')),
  (X'22222222222222222222222222222222', NULL, 'Demo Project - API Service', '/tmp/demo-api', 'cargo build', 'cargo run', NULL, NULL, datetime('now', '-20 days'), datetime('now', '-2 days')),
  (X'33333333333333333333333333333333', NULL, 'Demo Project - Mobile App', '/tmp/demo-mobile', 'flutter pub get', 'flutter run', NULL, NULL, datetime('now', '-10 days'), datetime('now', '-3 days'));

-- Insert demo tasks for E-Commerce project
-- NOTE: For the prototype, we use title prefix to distinguish Epic vs Story
-- Epic tasks have no parent_task_attempt (NULL)
-- Story tasks also have NULL for now (proper implementation needs task_attempt records)

-- Epic 1: User Authentication
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', NULL, X'11111111111111111111111111111111', 
   'Epic: User Authentication System', 
   'Complete authentication system including login, signup, password reset, and social auth integration',
   'inprogress', NULL, datetime('now', '-25 days'), datetime('now', '-1 hours')),
  
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab', NULL, X'11111111111111111111111111111111',
   'Story: Email/Password Login',
   'Implement traditional email and password login with validation',
   'done', NULL, datetime('now', '-24 days'), datetime('now', '-20 days')),
   
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac', NULL, X'11111111111111111111111111111111',
   'Story: Social Login (Google, GitHub)',
   'OAuth integration for Google and GitHub',
   'done', NULL, datetime('now', '-23 days'), datetime('now', '-18 days')),
   
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaad', NULL, X'11111111111111111111111111111111',
   'Story: Password Reset Flow',
   'Email-based password reset with secure tokens',
   'inprogress', NULL, datetime('now', '-22 days'), datetime('now', '-1 hours')),
   
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaae', NULL, X'11111111111111111111111111111111',
   'Story: JWT Token Management',
   'Implement refresh tokens and token rotation',
   'todo', NULL, datetime('now', '-21 days'), datetime('now', '-21 days')),
   
  (X'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaf', NULL, X'11111111111111111111111111111111',
   'Story: 2FA Implementation',
   'Two-factor authentication using TOTP',
   'todo', NULL, datetime('now', '-20 days'), datetime('now', '-20 days'));

-- Epic 2: Shopping Cart
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  (X'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', NULL, X'11111111111111111111111111111111',
   'Epic: Shopping Cart Features',
   'Complete shopping cart functionality with persistence and checkout integration',
   'todo', NULL, datetime('now', '-15 days'), datetime('now', '-2 days')),
   
  (X'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbc', NULL, X'11111111111111111111111111111111',
   'Story: Add/Remove Items',
   'Basic cart operations - add, remove, update quantity',
   'todo', NULL, datetime('now', '-14 days'), datetime('now', '-14 days')),
   
  (X'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbd', NULL, X'11111111111111111111111111111111',
   'Story: Cart Persistence',
   'Save cart state to database and local storage',
   'todo', NULL, datetime('now', '-13 days'), datetime('now', '-13 days')),
   
  (X'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbe', NULL, X'11111111111111111111111111111111',
   'Story: Price Calculation',
   'Calculate totals, taxes, and discounts',
   'todo', NULL, datetime('now', '-12 days'), datetime('now', '-12 days')),
   
  (X'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbcf', NULL, X'11111111111111111111111111111111',
   'Story: Checkout Integration',
   'Connect cart to payment gateway',
   'todo', NULL, datetime('now', '-11 days'), datetime('now', '-11 days'));

-- Epic 3: Product Catalog
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  (X'cccccccccccccccccccccccccccccccc', NULL, X'11111111111111111111111111111111',
   'Epic: Product Catalog Management',
   'Product listing, search, filtering, and detail pages',
   'inreview', NULL, datetime('now', '-10 days'), datetime('now', '-1 days')),
   
  (X'cccccccccccccccccccccccccccccccd', NULL, X'11111111111111111111111111111111',
   'Story: Product List View',
   'Grid/list view with pagination',
   'done', NULL, datetime('now', '-9 days'), datetime('now', '-7 days')),
   
  (X'ccccccccccccccccccccccccccccccce', NULL, X'11111111111111111111111111111111',
   'Story: Advanced Search',
   'Full-text search with filters',
   'inreview', NULL, datetime('now', '-8 days'), datetime('now', '-1 days')),
   
  (X'cccccccccccccccccccccccccccccccf', NULL, X'11111111111111111111111111111111',
   'Story: Product Detail Page',
   'Detailed product view with images and reviews',
   'done', NULL, datetime('now', '-7 days'), datetime('now', '-5 days'));

-- Insert some standalone tasks (not part of Epic)
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  (X'dddddddddddddddddddddddddddddddd', NULL, X'11111111111111111111111111111111',
   'Fix production bug: Cart items disappearing',
   'Users reporting cart items are lost after page refresh',
   'done', NULL, datetime('now', '-5 days'), datetime('now', '-3 days')),
   
  (X'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee', NULL, X'11111111111111111111111111111111',
   'Update dependencies to latest versions',
   'Security audit recommended updating React and other deps',
   'cancelled', NULL, datetime('now', '-4 days'), datetime('now', '-2 days'));

-- Insert tasks for API Service project
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  (X'ffffffffffffffffffffffffffffffff', NULL, X'22222222222222222222222222222222',
   'Epic: RESTful API Implementation',
   'Design and implement RESTful API endpoints',
   'inprogress', NULL, datetime('now', '-15 days'), datetime('now', '-1 hours')),
   
  ('ffffffff-ffff-ffff-ffff-ffffffffffff1', NULL, X'22222222222222222222222222222222',
   'Story: User CRUD Endpoints',
   'Create, Read, Update, Delete operations for users',
   'done', NULL, datetime('now', '-14 days'), datetime('now', '-10 days')),
   
  ('ffffffff-ffff-ffff-ffff-ffffffffffff2', NULL, X'22222222222222222222222222222222',
   'Story: Authentication Middleware',
   'JWT validation and authorization checks',
   'inprogress', NULL, datetime('now', '-13 days'), datetime('now', '-1 hours')),
   
  ('ffffffff-ffff-ffff-ffff-ffffffffffff3', NULL, X'22222222222222222222222222222222',
   'Story: API Documentation',
   'OpenAPI/Swagger documentation',
   'todo', NULL, datetime('now', '-12 days'), datetime('now', '-12 days'));

-- Insert tasks for Mobile App project
INSERT OR REPLACE INTO tasks (id, user_id, project_id, title, description, status, parent_task_attempt, created_at, updated_at)
VALUES 
  ('gggggggg-gggg-gggg-gggg-gggggggggggg', NULL, X'33333333333333333333333333333333',
   'Epic: Onboarding Flow',
   'User onboarding screens and tutorials',
   'todo', NULL, datetime('now', '-8 days'), datetime('now', '-8 days')),
   
  ('gggggggg-gggg-gggg-gggg-gggggggggggh', NULL, X'33333333333333333333333333333333',
   'Story: Welcome Screens',
   'Intro carousel with app features',
   'todo', NULL, datetime('now', '-7 days'), datetime('now', '-7 days')),
   
  ('gggggggg-gggg-gggg-gggg-ggggggggggi', NULL, X'33333333333333333333333333333333',
   'Story: Permission Requests',
   'Request camera, location, notifications permissions',
   'todo', NULL, datetime('now', '-6 days'), datetime('now', '-6 days'));

-- Note: Task attempts, execution processes, etc. are not included in this seed
-- as they require more complex setup. This provides enough data for UI development.
