provider "digitalocean" {
  token             = var.digitalocean_token
  spaces_access_id  = var.spaces_access_id
  spaces_secret_key = var.spaces_secret_key
}
