const net = require('net')

const INCOMING_PORT = 26000
const OUTGOING_PORT = 27017

function printBuffer(data, title) {
    console.info(`=== ${title} ===`)
    let display = ''
    let byte_count = 0
    for (const entry of data.entries()) {
        if (byte_count % 16 === 0) {
            display += (byte_count > 0 ? '\n' : '') + byte_count.toString(16).padStart(4, '0') + ' | '
        }
        display += entry[1].toString(16).padStart(2, '0') + ' '
        byte_count += 1
    }
    console.info(display)
    console.info('------------------------\n')
}

let socketToMongo
const socketToMongod = new net.Socket()

socketToMongod.connect(OUTGOING_PORT, '127.0.0.1', function() {
    console.log(`Socket connected to mongod at ${OUTGOING_PORT}`)
})

socketToMongod.on('data', data => {
    printBuffer(data, 'Mongo Daemon to Client')
    socketToMongo.write(data)
})

socketToMongod.on('error', error => {
    console.error('socket to mongod error:', error)
})

const proxyServer = net.createServer(socket => {
    // 'connection' listener
    socketToMongo = socket

    console.log('mongo client connected to proxy')

    socket.on('data', data => {
        printBuffer(data, 'Mongo Client to Daemon')
        socketToMongod.write(data)
    })

    socket.on('end', () => {
        console.log('mongo client disconnected from proxy')
    })
})

proxyServer.on('error', (err) => {
    throw err
})

proxyServer.listen(INCOMING_PORT, () => {
    console.log(`Proxy server bound to ${INCOMING_PORT}`)
})
