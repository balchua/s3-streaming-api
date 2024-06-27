

resource "digitalocean_spaces_bucket" "transitfiles" {
  name          = "transitfiles"
  region        = "sgp1"
  force_destroy = true
}
