
(function(){
  var key='vleo-guide-depth';
  function set(d){document.body.dataset.depth=d;
    document.querySelectorAll('[data-depth]').forEach(function(b){if(b.tagName==='BUTTON')b.classList.toggle('sel',b.dataset.depth===d);});
    try{localStorage.setItem(key,d);}catch(e){}}
  try{var s=localStorage.getItem(key);if(s)set(s);}catch(e){}
  document.querySelectorAll('button[data-depth]').forEach(function(b){b.onclick=function(){set(b.dataset.depth);};});
  var q=document.getElementById('q'),none=document.querySelector('.none');
  q.addEventListener('input',function(){var t=q.value.trim().toLowerCase(),shown=0;
    document.querySelectorAll('.sec').forEach(function(s){var hit=!t||s.dataset.text.indexOf(t)>=0||s.textContent.toLowerCase().indexOf(t)>=0;
      s.classList.toggle('hide',!hit);if(hit)shown++;});
    none.hidden=shown>0;});
  document.addEventListener('click',function(e){var b=e.target.closest&&e.target.closest('.copy');if(!b)return;
    var done=function(){b.textContent='copied';setTimeout(function(){b.textContent='copy';},1200);};
    if(navigator.clipboard)navigator.clipboard.writeText(b.dataset.copy).then(done,function(){});else done();});
})();
