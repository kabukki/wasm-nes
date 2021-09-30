export class Store {
    public database: string;
    public version: number;
    public store: string;

    constructor (database = 'retro', version = 1, store = 'nes') {
        this.database = database;
        this.version = version;
        this.store = store;
    }

    private init (): Promise<IDBDatabase> {
        return new Promise((resolve, reject) => {
            const database = window.indexedDB.open(this.database, this.version);
        
            database.onerror = (err) => {
                console.error(err);
                reject(err);
            };
            database.onsuccess = () => {
                resolve(database.result);
            };
            database.onupgradeneeded = () => {
                database.result.createObjectStore(this.store);
            };
        });
    }
    
    private request (mode: IDBTransactionMode, callback: (store: IDBObjectStore) => IDBRequest)  {
        return new Promise(async (resolve, reject) => {
            const db = await this.init();
            const store = db.transaction(this.store, mode).objectStore(this.store);
            const request = callback(store);
            request.onerror = reject;
            request.onsuccess = () => resolve(request.result);
        });
    }

    async save (key, value) {
        return this.request('readwrite', (store) => store.put(value, key));
    }

    async remove (key) {
        return this.request('readwrite', (store) => store.delete(key));
    }

    async get (key) {
        return this.request('readonly', (store) => store.get(key));
    }

    async getAll () {
        return this.request('readonly', (store) => store.getAll());
    }
}
