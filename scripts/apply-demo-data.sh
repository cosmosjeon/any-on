#!/bin/bash

# Apply Demo Data Script
# This script applies the demo data to the SQLite database

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}   Anyon Demo Data Setup${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Find the database file
DB_PATH=""
if [ -f "dev_assets/db.sqlite" ]; then
    DB_PATH="dev_assets/db.sqlite"
elif [ -f "db.sqlite" ]; then
    DB_PATH="db.sqlite"
elif [ -f "anyon.db" ]; then
    DB_PATH="anyon.db"
elif [ -f "../anyon.db" ]; then
    DB_PATH="../anyon.db"
elif [ -f "../dev_assets/db.sqlite" ]; then
    DB_PATH="../dev_assets/db.sqlite"
else
    echo -e "${RED}❌ Error: Could not find anyon.db${NC}"
    echo -e "${YELLOW}💡 Please run this script from the project root or scripts directory${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Found database at: ${DB_PATH}"

# Check if sqlite3 is installed
if ! command -v sqlite3 &> /dev/null; then
    echo -e "${RED}❌ Error: sqlite3 is not installed${NC}"
    echo -e "${YELLOW}💡 Please install sqlite3:${NC}"
    echo -e "   macOS: brew install sqlite3"
    echo -e "   Ubuntu: sudo apt-get install sqlite3"
    exit 1
fi

# Backup the database
BACKUP_PATH="${DB_PATH}.backup.$(date +%Y%m%d_%H%M%S)"
echo ""
echo -e "${BLUE}Creating backup...${NC}"
cp "$DB_PATH" "$BACKUP_PATH"
echo -e "${GREEN}✓${NC} Backup created: ${BACKUP_PATH}"

# Apply the seed data
echo ""
echo -e "${BLUE}Applying demo data...${NC}"
SQL_FILE="scripts/seed-demo-data.sql"
if [ ! -f "$SQL_FILE" ]; then
    SQL_FILE="seed-demo-data.sql"
fi

if [ ! -f "$SQL_FILE" ]; then
    echo -e "${RED}❌ Error: Could not find seed-demo-data.sql${NC}"
    exit 1
fi

sqlite3 "$DB_PATH" < "$SQL_FILE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC} Demo data applied successfully!"
else
    echo -e "${RED}❌ Error: Failed to apply demo data${NC}"
    echo -e "${YELLOW}💡 Restoring from backup...${NC}"
    cp "$BACKUP_PATH" "$DB_PATH"
    echo -e "${GREEN}✓${NC} Database restored from backup"
    exit 1
fi

# Show summary
echo ""
echo -e "${BLUE}======================================${NC}"
echo -e "${GREEN}✨ Demo Data Setup Complete!${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""
echo -e "${YELLOW}📊 Demo Data Summary:${NC}"
echo -e "   • 3 Projects (E-Commerce, API Service, Mobile App)"
echo -e "   • Multiple Epics with Stories"
echo -e "   • Tasks in various states (todo, inprogress, inreview, done)"
echo ""
echo -e "${YELLOW}🚀 Next Steps:${NC}"
echo -e "   1. Start the dev server: ${BLUE}pnpm run dev${NC}"
echo -e "   2. Navigate to the Kanban board"
echo -e "   3. Select 'Demo Project - E-Commerce'"
echo -e "   4. Explore the Epic-Story structure in the Backlog column"
echo ""
echo -e "${YELLOW}💡 Note:${NC}"
echo -e "   • Demo data has user_id = NULL (orphaned)"
echo -e "   • First GitHub login will claim this data"
echo -e "   • Backup saved at: ${BACKUP_PATH}"
echo ""
