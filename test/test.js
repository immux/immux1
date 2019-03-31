require('jest')
const fetch = require('isomorphic-fetch')

const db = {
    host: `localhost:1991`,
    async simple_get(collection, key) {
        const x = await fetch(`http://${this.host}/${collection}/${key}`)
        return await x.text()
    },
    async get_at_height(collection, key, height) {
        const x = await fetch(`http://${this.host}/${collection}/${key}?height=${height}`)
        return await x.text()
    },
    async set(collection, key, value) {
        const x = await fetch(`http://${this.host}/${collection}/${key}`, {
            method: 'PUT',
            body: value
        })
        return await x.text()
    },
    async revert_one(collection, key, height) {
        const x = await fetch(`http://${this.host}/${collection}/${key}?revert=${height}`, {
            method: 'PUT',
        })
        return await x.text()
    },
    async revert_all(height) {
        const x = await fetch(`http://${this.host}/?revert_all=${height}`, {
            method: 'PUT',
        })
        return await x.text()
    }
}

/*
test('single-document version tracking', async () => {
    const doc = 'a'
    const collection = 'collection'
    const dummyData = i => `test${i}`

    for (let i = 1; i <= 10; i += 1) {
        await db.set(collection, doc, dummyData(i))
    }

    for (let i = 1; i <= 10; i += 1) {
        let versioned = await db.get_at_height(collection, doc, i)
        expect(versioned).toBe(dummyData(i))
    }

    for (let i = 1; i <= 10; i += 1) {
        await db.revert_one(collection, doc, i)
        const current = await db.simple_get(collection, doc)
        expect(current).toBe(dummyData(i))
    }
})
*/

test('multi-document versioning', async () => {
    const collection = 'collection'
    const dummyData = (doc, i) => `data-${doc}-${i}`

    const docList = ['a', 'b', 'c']
    for (let i = 1; i <= 10; i += 1) {
        for (const doc of docList) {
            await db.set(collection, doc, dummyData(doc, i))
        }
    }

    await db.revert_one(collection, 'a', 1);
    expect(await db.simple_get(collection, 'a')).toBe(dummyData('a', 1))

    await db.revert_one(collection, 'b', 5);
    expect(await db.simple_get(collection, 'b')).toBe(dummyData('b', 2))

    expect(await db.get_at_height(collection, 'c', 6)).toBe(dummyData('c', 2))

    await db.revert_all(5)

    expect(await db.simple_get(collection, 'a')).toBe(await db.get_at_height(collection, 'a', 5))
    expect(await db.simple_get(collection, 'b')).toBe(await db.get_at_height(collection, 'b', 5))
    expect(await db.simple_get(collection, 'c')).toBe(await db.get_at_height(collection, 'c', 5))
})
