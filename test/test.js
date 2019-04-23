require('jest')
const fetch = require('isomorphic-fetch')

const db = {
    host: `localhost:1991`,
    async simpleGet(collection, key) {
        const response = await fetch(`http://${this.host}/${collection}/${key}`)
        return await response.text()
    },
    async getAtHeight(collection, key, height) {
        const response = await fetch(`http://${this.host}/${collection}/${key}?height=${height}`)
        return await response.text()
    },
    async set(collection, key, value) {
        const response = await fetch(`http://${this.host}/${collection}/${key}`, {
            method: 'PUT',
            body: value
        })
        return await response.text()
    },
    async revertOne(collection, key, height) {
        const response = await fetch(`http://${this.host}/${collection}/${key}?revert=${height}`, {
            method: 'PUT',
        })
        return await response.text()
    },
    async revertAll(height) {
        const response = await fetch(`http://${this.host}/?revert_all=${height}`, {
            method: 'PUT',
        })
        return await response.text()
    },
    async readNamespace() {
        const response = await fetch(`http://${this.host}/?chain`)
        return await response.text()
    },
    async switchNamespace(namespace) {
        const response = await fetch(`http://${this.host}/?chain=${namespace}`, {
            method: 'PUT',
        })
        return await response.text()
    },
}

test('changing database namespace', async () => {
    const doc = 'doc'
    const collection = 'collection'

    await db.switchNamespace('namespace-1')
    await db.set(collection, doc, '1')

    await db.switchNamespace('namespace-2')
    await db.set(collection, doc, '2')

    const dataInSpace2 = await db.simpleGet(collection, doc)
    expect(dataInSpace2).toBe('2')

    await db.switchNamespace('namespace-1')
    const dataInSpace1 = await db.simpleGet(collection, doc)
    expect(dataInSpace1).toBe('1')
})

test('single-document version tracking', async () => {
    await db.switchNamespace('single-document')

    const doc = 'a'
    const collection = 'collection'
    const dummyData = i => `test${i}`

    for (let i = 1; i <= 10; i += 1) {
        await db.set(collection, doc, dummyData(i))
    }

    for (let i = 1; i <= 10; i += 1) {
        let versioned = await db.getAtHeight(collection, doc, i)
        expect(versioned).toBe(dummyData(i))
    }

    for (let i = 1; i <= 10; i += 1) {
        await db.revertOne(collection, doc, i)
        const current = await db.simpleGet(collection, doc)
        expect(current).toBe(dummyData(i))
    }
})

test('multi-document versioning', async () => {
    await db.switchNamespace('multi-document')

    const collection = 'collection'
    const dummyData = (doc, i) => `data-${doc}-${i}`

    const docList = ['a', 'b', 'c']
    for (let i = 1; i <= 10; i += 1) {
        for (const doc of docList) {
            await db.set(collection, doc, dummyData(doc, i))
        }
    }

    await db.revertOne(collection, 'a', 1);
    expect(await db.simpleGet(collection, 'a')).toBe(dummyData('a', 1))

    await db.revertOne(collection, 'b', 5);
    expect(await db.simpleGet(collection, 'b')).toBe(dummyData('b', 2))

    expect(await db.getAtHeight(collection, 'c', 6)).toBe(dummyData('c', 2))

    await db.revertAll(5)

    expect(await db.simpleGet(collection, 'a')).toBe(await db.getAtHeight(collection, 'a', 5))
    expect(await db.simpleGet(collection, 'b')).toBe(await db.getAtHeight(collection, 'b', 5))
    expect(await db.simpleGet(collection, 'c')).toBe(await db.getAtHeight(collection, 'c', 5))
})
