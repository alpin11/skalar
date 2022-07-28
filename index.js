import axios from "axios"
import sharp from "sharp"
import Fastify from "fastify"
import mimeTypes from 'mime-types'

const allowedDomains = JSON.parse(process.env.ALLOWED_DOMAINS ?? "[]")

const fastify = Fastify()

fastify.get("/", {
  schema: {
    querystring: {
      url: {type: "string"},
      width: {type: "integer"},
      height: {type: "integer"},
      format: {type: "string"},
      quality: {type: "string"},
    }
  }}, async (req, res) => {
    if (!req.query.url) {
      return "url, width, height query parameters required"
    }

    // Check if domain whitelisting is enabled and if the domain is whitelisted
    if ((allowedDomains.length > 0) && (!allowedDomains.includes(new URL(req.query.url).hostname))) {
      return "Domain Name not allowed!"
    }


    const input = (await axios({ url: encodeURI(req.query.url), responseType: "arraybuffer" })).data

    const quality = req.query.quality ? parseInt(req.query.quality) : 80

    const output = await sharp(input).resize({width: req.query.width, height: req.query.height, fit: "cover"}).toFormat(req.query.format ?? "webp", {quality}).toBuffer()

    const mimeType = mimeTypes.lookup(req.query.format) || "image/webp"

    console.log(`serving ${req.query.url}`)
    res.type(mimeType).code(200)
    return output
})

fastify.listen({port: 8020, host: "0.0.0.0"}, (err, addr) => {
  if (err)
    console.log(err)
})

console.log("starting on http://localhost:8020")
console.log("allowed domains", allowedDomains)
