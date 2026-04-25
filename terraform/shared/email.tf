# ProtonMail — MX routing
resource "aws_route53_record" "mx" {
  zone_id = aws_route53_zone.main.zone_id
  name    = var.domain
  type    = "MX"
  ttl     = 3600
  records = [
    "10 mail.protonmail.ch.",
    "20 mailsec.protonmail.ch.",
  ]
}

# ProtonMail — domain verification + SPF (single TXT record with both values)
resource "aws_route53_record" "txt_root" {
  zone_id = aws_route53_zone.main.zone_id
  name    = var.domain
  type    = "TXT"
  ttl     = 3600
  records = [
    "protonmail-verification=c0f936e61673034574c6910deeb4552308a1d3d3",
    "v=spf1 include:_spf.protonmail.ch ~all",
  ]
}

# ProtonMail — DMARC
resource "aws_route53_record" "dmarc" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "_dmarc.${var.domain}"
  type    = "TXT"
  ttl     = 3600
  records = ["v=DMARC1; p=quarantine; adkim=r; aspf=r; rua=mailto:dmarc_rua@onsecureserver.net;"]
}

# ProtonMail — DKIM keys (3 selectors)
resource "aws_route53_record" "dkim1" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "protonmail._domainkey.${var.domain}"
  type    = "CNAME"
  ttl     = 3600
  records = ["protonmail.domainkey.dsb5npx2yo3xu6ymfc6xobjggcdulkgxb3ogpayyds6cb553sm2ga.domains.proton.ch."]
}

resource "aws_route53_record" "dkim2" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "protonmail2._domainkey.${var.domain}"
  type    = "CNAME"
  ttl     = 3600
  records = ["protonmail2.domainkey.dsb5npx2yo3xu6ymfc6xobjggcdulkgxb3ogpayyds6cb553sm2ga.domains.proton.ch."]
}

resource "aws_route53_record" "dkim3" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "protonmail3._domainkey.${var.domain}"
  type    = "CNAME"
  ttl     = 3600
  records = ["protonmail3.domainkey.dsb5npx2yo3xu6ymfc6xobjggcdulkgxb3ogpayyds6cb553sm2ga.domains.proton.ch."]
}
