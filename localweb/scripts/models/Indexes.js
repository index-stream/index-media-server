class Indexes {
    constructor() {
        this._indexes = null;
    }

    // Set the indexes data
    setIndexes(indexes) {
        this._indexes = indexes;
        console.log('Indexes updated:', this._indexes);
    }

    // Get indexes
    getIndexes() {
        return this._indexes || [];
    }

    // Add a new index (local update)
    addIndex(index) {
        if (this._indexes) {
            if (!this._indexes) {
                this._indexes = [];
            }
            this._indexes.push(index);
            console.log('Index added locally:', index);
        }
    }

    // Update an existing index (local update)
    updateIndex(updatedIndex) {
        if (this._indexes) {
            const index = this._indexes.findIndex(i => i.id === updatedIndex.id);
            if (index !== -1) {
                this._indexes[index] = updatedIndex;
                console.log('Index updated locally:', updatedIndex);
            }
        }
    }

    // Remove an index (local update)
    removeIndex(indexId) {
        if (this._indexes) {
            const index = this._indexes.findIndex(i => i.id === indexId);
            if (index !== -1) {
                const removedIndex = this._indexes.splice(index, 1)[0];
                console.log('Index removed locally:', removedIndex);
            }
        }
    }

    // Check if indexes are loaded
    isLoaded() {
        return this._indexes !== null;
    }

    // Clear indexes (useful for logout/reset)
    clear() {
        this._indexes = null;
        console.log('Indexes cleared');
    }
}

// Create a singleton instance
let indexes = new Indexes();
export default indexes;
