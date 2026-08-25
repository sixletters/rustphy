from abc import ABC, abstractmethod

class BaseHashMap(ABC):
    @abstractmethod
    def set(key, value):
        pass

    @abstractmethod
    def get(key):
        pass

    @abstractmethod
    def has(key):
        pass

    @abstractmethod
    def delete(key):
        pass
