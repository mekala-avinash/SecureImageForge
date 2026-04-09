"""
Database connection module for SecureImage Forge
"""
import os
from motor.motor_asyncio import AsyncIOMotorClient

# MongoDB connection
mongo_url = os.environ['MONGO_URL']
client = AsyncIOMotorClient(mongo_url)
db = client[os.environ['DB_NAME']]


def get_db():
    """Get database instance"""
    return db


def close_db():
    """Close database connection"""
    client.close()
